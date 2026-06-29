use crate::model::{load_all_beds, load_live_seeds, Bed, Garden};
use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;

/// Search for a phrase across the garden. Like `grep` but the binary
/// owns the experience: matching seeds with context, mood, tags, bed.
#[derive(Args, Debug)]
pub struct ForageArgs {
    /// Phrase to search for (case-insensitive substring).
    pub query: String,

    /// Lines of context before and after each match (default 1).
    #[arg(long, default_value_t = 1)]
    pub context: usize,

    /// Filter to seeds in a specific bed (by slug or name).
    #[arg(long)]
    pub bed: Option<String>,

    /// Include composted seeds.
    #[arg(long)]
    pub all: bool,

    /// Output as JSON instead of a human-readable list.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ForageArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = if args.all {
        crate::model::load_all_seeds(&garden)?
    } else {
        load_live_seeds(&garden)?
    };
    let beds = load_all_beds(&garden)?;
    let query_lower = args.query.to_lowercase();

    let mut hits: Vec<(&crate::model::Seed, Vec<(usize, &str)>)> = Vec::new();
    for s in &seeds {
        if let Some(b) = args.bed.as_deref() {
            let in_bed = beds
                .iter()
                .find(|bed| Bed::slug(&bed.name) == Bed::slug(b))
                .map(|bed| bed.seeds.contains(&s.id))
                .unwrap_or(false);
            if !in_bed {
                continue;
            }
        }
        let lines: Vec<&str> = s.body.lines().collect();
        let mut matches = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                matches.push((i, *line));
            }
        }
        if !matches.is_empty() {
            hits.push((s, matches));
        }
    }

    if args.json {
        let json: Vec<serde_json::Value> = hits
            .iter()
            .map(|(s, matches)| {
                let lines: Vec<&str> = s.body.lines().collect();
                let snippets: Vec<serde_json::Value> = matches
                    .iter()
                    .map(|(line_idx, line)| {
                        let start = line_idx.saturating_sub(args.context);
                        let end = (line_idx + args.context + 1).min(lines.len());
                        let snippet_lines: Vec<String> = (start..end)
                            .map(|i| lines[i].to_string())
                            .collect();
                        serde_json::json!({
                            "match_line": line_idx + 1,
                            "match_text": line,
                            "context": snippet_lines.join("\n"),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "id": s.id,
                    "planted": s.planted.to_string(),
                    "mood": s.mood,
                    "tags": s.tags,
                    "is_composted": s.is_composted,
                    "matches": snippets,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("serializing forage")
        );
        return Ok(());
    }

    if hits.is_empty() {
        println!(
            "{}",
            format!("no seeds contain `{}`.", args.query).yellow()
        );
        return Ok(());
    }

    println!(
        "{} {} match{} in {} seed{}",
        "·".green(),
        hits.iter().map(|(_, m)| m.len()).sum::<usize>(),
        if hits.iter().map(|(_, m)| m.len()).sum::<usize>() == 1 {
            ""
        } else {
            "es"
        },
        hits.len(),
        if hits.len() == 1 { "" } else { "s" }
    );
    println!();
    for (s, matches) in &hits {
        let marker = if s.is_composted { "~" } else { " " };
        let mood_part = s
            .mood
            .as_deref()
            .map(|m| format!(" {}", m.italic()))
            .unwrap_or_default();
        println!(
            "  {} {} {}{}",
            marker.dimmed(),
            s.planted.to_string().dimmed(),
            s.id.bright_green(),
            mood_part
        );
        let lines: Vec<&str> = s.body.lines().collect();
        for (line_idx, matched_line) in matches {
            let start = line_idx.saturating_sub(args.context);
            let end = (line_idx + args.context + 1).min(lines.len());
            for i in start..end {
                let prefix = if i == *line_idx { "    >" } else { "     " };
                if i == *line_idx {
                    // Highlight the matching substring (case-insensitive).
                    let lower = matched_line.to_lowercase();
                    let mut out = String::new();
                    let mut idx = 0;
                    while let Some(pos) = lower[idx..].find(&query_lower) {
                        out.push_str(&matched_line[idx..idx + pos]);
                        let end_idx = idx + pos + query_lower.len();
                        let matched = matched_line[idx + pos..end_idx].to_string();
                        out.push_str(&matched.bright_yellow().to_string());
                        idx = end_idx;
                    }
                    out.push_str(&matched_line[idx..]);
                    println!("{} {}", prefix.dimmed(), out);
                } else {
                    println!("{} {}", prefix.dimmed(), matched_line);
                }
            }
            println!();
        }
    }
    Ok(())
}
