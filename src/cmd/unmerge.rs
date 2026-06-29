use crate::model::{load_all_seeds, Garden, Seed};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;

/// Reverse a `hortus merge`. Splits a merged seed back into its two originals
/// (which must be in compost) and deletes the merged seed.
#[derive(Args, Debug)]
pub struct UnmergeArgs {
    /// Id of the merged seed to split.
    pub seed: String,
}

pub fn run(args: UnmergeArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let all = load_all_seeds(&garden)?;

    let merged = all
        .iter()
        .find(|s| s.id == args.seed)
        .with_context(|| format!("no seed named `{}`", args.seed))?;

    let bodies = parse_merge_bodies(&merged.body)
        .with_context(|| "could not parse the merged seed's body sections")?;

    if bodies.len() != 2 {
        bail!("expected 2 sections in merged body, found {}", bodies.len());
    }

    // Restore each original from compost, or error if it's not there.
    let today = Utc::now().date_naive();
    for (id, body) in &bodies {
        let compost_path = garden.compost_dir().join(format!("{}.md", id));
        let live_path = garden.seeds_dir().join(format!("{}.md", id));
        if live_path.exists() {
            bail!(
                "refusing to unmerge: `{}` already exists in the living garden",
                id
            );
        }
        if !compost_path.exists() {
            bail!(
                "refusing to unmerge: original `{}` is not in compost (was it deleted?)",
                id
            );
        }
        let mut s = Seed::load(&compost_path)?;
        s.is_composted = false;
        s.composted_at = None;
        s.epitaph = None;
        s.body = body.clone();
        s.last_tended = Some(today);
        s.save(&garden)?;
        println!("{} restored {}", "·".green(), id.bright_green());
    }

    // Delete the merged seed.
    let merged_path = garden.seeds_dir().join(format!("{}.md", merged.id));
    if merged_path.exists() {
        std::fs::remove_file(&merged_path).context("deleting merged seed")?;
    }
    println!(
        "{} removed merged seed {}",
        "·".green(),
        merged.id.bright_green()
    );
    Ok(())
}

/// Parse `## from <id>\n\n<body>` sections. Returns Vec of (id, body) pairs.
fn parse_merge_bodies(body: &str) -> Option<Vec<(String, String)>> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_body = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## from ") {
            if let Some(prev) = current_id.take() {
                sections.push((prev, current_body.trim().to_string()));
            }
            current_id = Some(rest.trim().to_string());
            current_body = String::new();
        } else if current_id.is_some() {
            if !current_body.is_empty() {
                current_body.push('\n');
            }
            current_body.push_str(line);
        }
    }
    if let Some(prev) = current_id {
        sections.push((prev, current_body.trim().to_string()));
    }
    if sections.is_empty() {
        None
    } else {
        Some(sections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_merge_bodies_two_sections() {
        let body = "## from alpha\nfirst body\n\n## from beta\nsecond body\n";
        let sections = parse_merge_bodies(body).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].0, "alpha");
        assert_eq!(sections[0].1, "first body");
        assert_eq!(sections[1].0, "beta");
        assert_eq!(sections[1].1, "second body");
    }

    #[test]
    fn parse_merge_bodies_single_section() {
        let body = "## from only\nbody text\n";
        let sections = parse_merge_bodies(body).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].0, "only");
        assert_eq!(sections[0].1, "body text");
    }

    #[test]
    fn parse_merge_bodies_none() {
        assert!(parse_merge_bodies("no headers here").is_none());
    }
}
