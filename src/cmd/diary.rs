use crate::model::{load_live_seeds, Garden, Seed};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::Args;
use owo_colors::OwoColorize;
use std::fs;

/// Write a weekly diary entry from this week's seeds.
#[derive(Args, Debug)]
pub struct DiaryArgs {
    /// Number of days to include (default 7).
    #[arg(long, default_value_t = 7)]
    pub days: i64,

    /// Don't write to disk; only print to stdout.
    #[arg(long)]
    pub stdout: bool,
}

pub fn run(args: DiaryArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let today = Utc::now().date_naive();
    let since = today - Duration::days(args.days);
    let seeds = load_live_seeds(&garden)?;
    let climate = crate::model::Climate::load_or_default(&garden)?;

    let recent: Vec<&Seed> = seeds.iter().filter(|s| s.planted >= since).collect();

    let diary = render(&recent, since, today, &climate, args.days);

    if args.stdout {
        println!("{}", diary);
    } else {
        let dir = garden.root.join("diary");
        fs::create_dir_all(&dir).context("creating diary dir")?;
        let path = dir.join(format!("{}-diary.md", today));
        fs::write(&path, &diary).context("writing diary")?;
        println!(
            "{} wrote {} ({} seed{} over {} day{})",
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

fn render(seeds: &[&Seed], since: chrono::NaiveDate, until: chrono::NaiveDate, climate: &crate::model::Climate, days: i64) -> String {
    use std::collections::BTreeMap;
    let mut by_day: BTreeMap<chrono::NaiveDate, Vec<&Seed>> = BTreeMap::new();
    for s in seeds {
        by_day.entry(s.planted).or_default().push(*s);
    }

    let mut out = String::new();
    out.push_str(&format!("# Diary: {} → {}\n\n", since, until));
    let mut meta = format!("*{} day{} · {} seed{}", days, if days == 1 { "" } else { "s" }, seeds.len(), if seeds.len() == 1 { "" } else { "s" });
    if let Some(m) = climate.now.mood.as_deref() {
        meta.push_str(&format!(" · mood: {}", m));
    }
    if let Some(r) = climate.now.reading.as_deref() {
        meta.push_str(&format!(" · reading: {}", r));
    }
    out.push_str(&meta);
    out.push_str("\n\n");

    for (day, ss) in by_day.iter().rev() {
        out.push_str(&format!("## {}\n\n", day));
        for s in ss {
            let mood_part = s
                .mood
                .as_deref()
                .map(|m| format!(" · {}", m))
                .unwrap_or_default();
            out.push_str(&format!(
                "- **{}**{}\n",
                s.id, mood_part
            ));
            for line in s.body.lines().take(8) {
                if line.trim().is_empty() {
                    continue;
                }
                out.push_str(&format!("  > {}\n", line));
            }
            out.push('\n');
        }
    }

    if seeds.is_empty() {
        out.push_str("*nothing planted in this window.*\n");
    }

    out
}
