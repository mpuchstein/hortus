use crate::model::{load_live_seeds, make_seed_id, unique_seed_id, Garden, Seed};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;
use std::fs;
use std::io::{self, Read};
use std::process::Command;

/// Plant a new seed. The thought is a single argument, or stdin via `--stdin` / `-`.
/// With no args, opens your editor with a template that includes the current
/// climate and your last few seeds — planting becomes reflection.
#[derive(Args, Debug)]
pub struct PlantArgs {
    /// The thought to plant. Pass `-` to read from stdin.
    /// If omitted, opens the editor with a planting template.
    pub thought: Vec<String>,

    /// Read the thought from stdin.
    #[arg(long)]
    pub stdin: bool,

    /// Open the editor with a template, even if a thought was given.
    #[arg(long)]
    pub interactive: bool,

    /// Optional mood tag for the climate.
    #[arg(long)]
    pub mood: Option<String>,

    /// Optional tags (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub tag: Vec<String>,
}

pub fn run(args: PlantArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let interactive = args.interactive || (args.thought.is_empty() && !args.stdin);

    // ----- Interactive mode: open editor with a template. -----
    if interactive {
        return run_interactive(&garden, &args);
    }

    // ----- One-shot mode: read body, write seed. -----
    let body = if args.stdin || args.thought.first().map(String::as_str) == Some("-") {
        let mut s = String::new();
        io::stdin()
            .read_to_string(&mut s)
            .context("reading thought from stdin")?;
        s
    } else {
        args.thought.join(" ")
    };

    let body = body.trim();
    if body.is_empty() {
        anyhow::bail!("a seed needs a thought. try: hortus plant \"a small idea\"");
    }

    let planted = Utc::now().date_naive();
    let base = make_seed_id(planted, body);
    let id = unique_seed_id(&garden, &base);

    let seed = Seed {
        id: id.clone(),
        planted,
        last_tended: Some(planted),
        mood: args.mood,
        tags: args.tag,
        composted_at: None,
        epitaph: None,
        body: body.to_string(),
        is_composted: false,
    };
    seed.save(&garden)?;

    println!("{} planted {}", "·".green(), id.bright_green());
    println!(
        "  {}",
        garden.seeds_dir().join(format!("{}.md", id)).display()
    );
    Ok(())
}

fn run_interactive(garden: &Garden, args: &PlantArgs) -> Result<()> {
    let today = Utc::now().date_naive();
    let climate = crate::model::Climate::load_or_default(garden)?;
    let recent = load_live_seeds(garden)?;
    let preview_id = make_seed_id(today, "draft-thought");

    let mut template = String::new();
    template.push_str("---\n");
    template.push_str(&format!("id: {}\n", preview_id));
    template.push_str(&format!("planted: {}\n", today));
    template.push_str(&format!("last_tended: {}\n", today));
    template.push_str(&format!(
        "mood: {}\n",
        args.mood
            .clone()
            .or_else(|| climate.now.mood.clone())
            .unwrap_or_default()
    ));
    template.push_str("tags: []\n");
    template.push_str("---\n\n");

    template.push_str("# climate at planting\n");
    if let Some(m) = climate.now.mood.as_deref() {
        template.push_str(&format!("mood: {}\n", m));
    } else {
        template.push_str("mood: (unset)\n");
    }
    if let Some(r) = climate.now.reading.as_deref() {
        template.push_str(&format!("reading: {}\n", r));
    }
    if let Some(s) = climate.now.season.as_deref() {
        template.push_str(&format!("season: {}\n", s));
    }
    template.push('\n');

    if !recent.is_empty() {
        template.push_str("# recent seeds (for context)\n");
        for s in recent.iter().rev().take(5) {
            template.push_str(&format!("- {}: {}\n", s.planted, s.id));
        }
        template.push('\n');
    }

    template.push_str("# the new thought\n");
    template.push_str("\n<!-- write below this line; the marker above will be removed -->\n\n");

    // Write to temp file.
    let mut f = tempfile::Builder::new()
        .prefix("hortus-plant-")
        .suffix(".md")
        .tempfile()
        .context("creating temp file")?;
    use std::io::Write;
    f.write_all(template.as_bytes())?;
    let path = f.path().to_path_buf();

    let editor_cmd = editor_command();
    let status = Command::new(editor_cmd.first().expect("editor resolved"))
        .args(&editor_cmd[1..])
        .arg(&path)
        .status()
        .context("launching editor")?;
    if !status.success() {
        anyhow::bail!("editor exited with {}", status);
    }

    let raw = fs::read_to_string(&path).context("reading edited file")?;
    let (fm, body) = crate::model::split_frontmatter(&raw);

    // Parse the frontmatter as a partial Seed.
    let mut s: Seed =
        serde_yaml::from_str(&fm).context("parsing frontmatter (check YAML syntax)")?;
    // The marker line separates template scaffolding from the user's text.
    // Take whatever is *after* the marker — that's the thought.
    let user_text = body
        .split_once("<!-- write below this line; the marker above will be removed -->")
        .map(|(_, after)| after)
        .unwrap_or("")
        .trim_start_matches('\n')
        .trim_end()
        .to_string();
    if user_text.is_empty() {
        anyhow::bail!("no thought was written. plant again when ready.");
    }
    let body = user_text;

    s.body = body;
    s.last_tended = Some(today);
    if s.planted != today {
        s.planted = today;
    }
    // Merge any CLI-supplied tags.
    let mut all_tags = s.tags.clone();
    all_tags.extend(args.tag.iter().cloned());
    all_tags.sort();
    all_tags.dedup();
    s.tags = all_tags;
    if s.mood.is_none() {
        s.mood = args.mood.clone();
    }
    s.composted_at = None;
    s.epitaph = None;
    s.is_composted = false;
    // Make sure the id is unique.
    s.id = unique_seed_id(garden, &s.id);

    s.save(garden)?;
    println!("{} planted {}", "·".green(), s.id.bright_green());
    println!(
        "  {}",
        garden.seeds_dir().join(format!("{}.md", s.id)).display()
    );
    Ok(())
}

fn editor_command() -> Vec<String> {
    let raw = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    simple_split(&raw)
}

fn simple_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for c in s.chars() {
        match c {
            '"' => in_quote = !in_quote,
            c if c.is_whitespace() && !in_quote => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        vec![s.to_string()]
    } else {
        out
    }
}
