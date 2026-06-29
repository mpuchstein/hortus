use crate::model::{load_all_beds, load_live_seeds, Bed, Garden, Seed};
use anyhow::Result;
use chrono::NaiveDate;
use clap::Args;
use owo_colors::OwoColorize;

/// List seeds, with optional filters.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter to a specific bed (by slug or name).
    #[arg(long)]
    pub bed: Option<String>,
    /// Filter by tag.
    #[arg(long)]
    pub tag: Option<String>,
    /// Filter by mood.
    #[arg(long)]
    pub mood: Option<String>,
    /// Only seeds planted on or after this date (YYYY-MM-DD or Nd/Nw ago).
    #[arg(long)]
    pub since: Option<String>,
    /// Include composted seeds.
    #[arg(long)]
    pub all: bool,
    /// Show only the first line of each body.
    #[arg(long)]
    pub oneline: bool,
    /// Output as JSON instead of a human-readable table.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ListArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let all_seeds = if args.all {
        crate::model::load_all_seeds(&garden)?
    } else {
        load_live_seeds(&garden)?
    };
    let beds = load_all_beds(&garden)?;

    if all_seeds.is_empty() {
        println!("{}", "the garden is empty.".yellow());
        return Ok(());
    }

    let since = args.since.as_deref().and_then(parse_since);

    let mut filtered: Vec<&Seed> = all_seeds
        .iter()
        .filter(|s| {
            if let Some(b) = args.bed.as_deref() {
                let in_bed = beds
                    .iter()
                    .find(|bed| Bed::slug(&bed.name) == Bed::slug(b))
                    .map(|bed| bed.seeds.contains(&s.id))
                    .unwrap_or(false);
                if !in_bed {
                    return false;
                }
            }
            if let Some(t) = args.tag.as_deref() {
                if !s.tags.iter().any(|x| x == t) {
                    return false;
                }
            }
            if let Some(m) = args.mood.as_deref() {
                if s.mood.as_deref() != Some(m) {
                    return false;
                }
            }
            if let Some(d) = since {
                if s.planted < d {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("{}", "no seeds match.".yellow());
        }
        return Ok(());
    }

    // Sort: live first, then by date desc
    filtered.sort_by(|a, b| {
        b.planted
            .cmp(&a.planted)
            .then(a.is_composted.cmp(&b.is_composted))
            .then(a.id.cmp(&b.id))
    });

    if args.json {
        let json: Vec<serde_json::Value> = filtered
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "planted": s.planted.to_string(),
                    "last_tended": s.last_tended.map(|d| d.to_string()),
                    "mood": s.mood,
                    "tags": s.tags,
                    "is_composted": s.is_composted,
                    "composted_at": s.composted_at.map(|d| d.to_string()),
                    "epitaph": s.epitaph,
                    "body": s.body.trim(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("serializing seeds")
        );
        return Ok(());
    }

    let n = filtered.len();
    let noun = if n == 1 { "seed" } else { "seeds" };
    println!("{} {} {}", "·".green(), n.to_string().bright_green(), noun);
    println!();
    for s in filtered {
        let marker = if s.is_composted { "~" } else { " " };
        let date = s.planted.to_string();
        let mood_part = s
            .mood
            .as_deref()
            .map(|m| format!(" {}", m.italic()))
            .unwrap_or_default();
        let tag_part = if s.tags.is_empty() {
            String::new()
        } else {
            format!(
                "  {}",
                s.tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        println!(
            "  {} {} {}{}{}",
            marker.dimmed(),
            date.dimmed(),
            s.id.bright_green(),
            mood_part,
            tag_part.dimmed()
        );
        if !args.oneline {
            if let Some(line) = s.body.lines().find(|l| !l.trim().is_empty()) {
                println!("      {}", line.dimmed());
            }
        }
    }

    Ok(())
}

fn parse_since(s: &str) -> Option<NaiveDate> {
    use chrono::Duration;
    let today = chrono::Utc::now().date_naive();
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    if let Some(num) = s.strip_suffix('d') {
        if let Ok(n) = num.parse::<i64>() {
            return Some(today - Duration::days(n));
        }
    }
    if let Some(num) = s.strip_suffix('w') {
        if let Ok(n) = num.parse::<i64>() {
            return Some(today - Duration::weeks(n));
        }
    }
    if let Some(num) = s.strip_suffix('m') {
        if let Ok(n) = num.parse::<i64>() {
            return Some(today - Duration::days(n * 30));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_iso() {
        let d = parse_since("2026-01-15").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
    }

    #[test]
    fn parse_since_invalid() {
        assert!(parse_since("not a date").is_none());
    }

    #[test]
    fn parse_since_days() {
        let d = parse_since("7d").unwrap();
        let expected = chrono::Utc::now().date_naive() - chrono::Duration::days(7);
        assert_eq!(d, expected);
    }
}
