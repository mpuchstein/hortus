use crate::model::{load_live_seeds, Garden, Seed};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::Args;
use owo_colors::OwoColorize;
use std::fs;

/// Write a one-page letter to self from the past month's seeds.
#[derive(Args, Debug)]
pub struct LetterArgs {
    /// Number of days to look back (default 30).
    #[arg(long, default_value_t = 30)]
    pub days: i64,

    /// Don't write to disk; only print to stdout.
    #[arg(long)]
    pub stdout: bool,
}

pub fn run(args: LetterArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let today = Utc::now().date_naive();
    let since = today - Duration::days(args.days);
    let seeds = load_live_seeds(&garden)?;
    let climate = crate::model::Climate::load_or_default(&garden)?;

    let recent: Vec<&Seed> = seeds.iter().filter(|s| s.planted >= since).collect();

    let letter = render(&recent, since, today, &climate);

    if args.stdout {
        println!("{}", letter);
    } else {
        let dir = garden.root.join("letters");
        fs::create_dir_all(&dir).context("creating letters dir")?;
        let path = dir.join(format!("{}-letter.md", today));
        fs::write(&path, &letter).context("writing letter")?;
        println!(
            "{} wrote {} ({} seed{} across {} day{})",
            "·".green(),
            path.display().to_string().bright_cyan(),
            recent.len(),
            if recent.len() == 1 { "" } else { "s" },
            args.days,
            if args.days == 1 { "" } else { "s" }
        );
    }
    Ok(())
}

fn first_sentence(s: &Seed) -> String {
    let first_line = s
        .body
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    // Within the first line, find the first complete sentence.
    let chars: Vec<(usize, char)> = first_line.char_indices().collect();
    let mut end = first_line.len();
    for (i, c) in &chars {
        if *c == '.' || *c == '?' || *c == '!' {
            end = i + c.len_utf8();
            break;
        }
    }
    first_line[..end].to_string()
}

fn rest_excerpt(s: &Seed) -> String {
    s.body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .skip(1)
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
}

fn pick_across_window<'a>(seeds: &[&'a Seed], count: usize) -> Vec<&'a Seed> {
    if seeds.is_empty() {
        return Vec::new();
    }
    if seeds.len() <= count {
        return seeds.to_vec();
    }
    // Pick seeds roughly evenly across the time window.
    let mut sorted: Vec<&Seed> = seeds.to_vec();
    sorted.sort_by_key(|s| s.planted);
    let mut out: Vec<&Seed> = Vec::with_capacity(count);
    for i in 0..count {
        let idx = i * (sorted.len() - 1) / (count - 1);
        out.push(sorted[idx]);
    }
    out
}

fn render(
    seeds: &[&Seed],
    since: chrono::NaiveDate,
    until: chrono::NaiveDate,
    climate: &crate::model::Climate,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Letter to self — {}\n\n", until));
    out.push_str(&format!(
        "*Composed from {} seed{} in the past {} days ({} → {}).*\n\n",
        seeds.len(),
        if seeds.len() == 1 { "" } else { "s" },
        (until - since).num_days(),
        since,
        until
    ));

    out.push_str("Dear me,\n\n");

    // Opening: speak the mood and the reading.
    let mood = climate.now.mood.as_deref().unwrap_or("here");
    let reading = climate.now.reading.as_deref();
    let n = seeds.len();

    let opening = match (n, reading) {
        (0, _) => "The garden has been quiet. I notice that I haven't planted anything since we last spoke, and I want to remember that quiet as a kind of growing too.".to_string(),
        (_, Some(r)) => format!(
            "I notice you've been {} this past while. I've been reading {}, and the weather of the garden has been the same — slow, a little tender, mostly just paying attention.",
            mood, r
        ),
        (_, None) => format!(
            "I notice you've been {} this past while. The garden has been the same weather — slow, a little tender, mostly just paying attention.",
            mood
        ),
    };
    out.push_str(&opening);
    out.push_str("\n\n");

    // Middle: reference 2-3 seeds.
    let picks = pick_across_window(seeds, 3);
    for (i, s) in picks.iter().enumerate() {
        let first = first_sentence(s);
        let rest = rest_excerpt(s);
        let paragraph = match i {
            0 => format!(
                "On {}, you wrote this and I want to keep it: *\"{}\"* The day I planted it, the rest of the seed was: {}",
                s.planted, first, truncate(&rest, 220)
            ),
            1 => format!(
                "A few days later, on {}, you came back to a related thought: *\"{}\"* I read it again now and the connection is: {}",
                s.planted, first, truncate(&rest, 200)
            ),
            _ => format!(
                "And most recently, on {}, you put down: *\"{}\"* That one I'm leaving exactly as it is.",
                s.planted, first
            ),
        };
        out.push_str(&paragraph);
        out.push_str("\n\n");
    }

    // Closing: a wish, varies with mood.
    let closing = if n == 0 {
        "When you're ready, plant something small. — me".to_string()
    } else {
        let m = climate.now.mood.as_deref().unwrap_or("");
        let c = match m {
            m if m.contains("quiet") => "tend what is already growing",
            m if m.contains("restless") => "plant something small and let it lie",
            m if m.contains("tender") => "be gentle with the next thing you write",
            m if m.contains("curious") => "follow the next question even if it leads nowhere",
            m if m.contains("hopeful") => "trust the seeds that are slow to sprout",
            _ => "trust the next sentence, even the half-formed one",
        };
        format!("I want to leave you with this: {}. — me", c)
    };
    out.push_str(&closing);
    out.push_str("\n\n");

    out.push_str(&format!("_{}, from the garden_\n", until));
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let cut = s[..max].rfind(' ').unwrap_or(max);
    format!("{}…", &s[..cut])
}
