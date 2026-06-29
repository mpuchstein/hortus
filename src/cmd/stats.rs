use crate::model::{load_all_beds, load_all_seeds, Garden};
use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use clap::Args;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;

/// A small annual review of the garden. Seeds per month, mood
/// distribution, beds that grew, compost history.
#[derive(Args, Debug)]
pub struct StatsArgs {
    /// Output as JSON instead of a human-readable summary.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: StatsArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = load_all_seeds(&garden)?;
    let live: Vec<_> = seeds.iter().filter(|s| !s.is_composted).collect();
    let composted: Vec<_> = seeds.iter().filter(|s| s.is_composted).collect();
    let beds = load_all_beds(&garden)?;

    // Per-month count, by year.
    let mut per_month: BTreeMap<(i32, u32), usize> = BTreeMap::new();
    for s in &live {
        let key = (s.planted.year(), s.planted.month());
        *per_month.entry(key).or_default() += 1;
    }
    // Mood histogram (live only).
    let mut moods: BTreeMap<String, usize> = BTreeMap::new();
    for s in &live {
        if let Some(m) = &s.mood {
            *moods.entry(m.clone()).or_default() += 1;
        }
    }
    // Tag histogram (live only).
    let mut tags: BTreeMap<String, usize> = BTreeMap::new();
    for s in &live {
        for t in &s.tags {
            *tags.entry(t.clone()).or_default() += 1;
        }
    }
    // Compost history.
    let mut compost_per_month: BTreeMap<(i32, u32), usize> = BTreeMap::new();
    for s in &composted {
        if let Some(d) = s.composted_at {
            let key = (d.year(), d.month());
            *compost_per_month.entry(key).or_default() += 1;
        }
    }
    // First and last dates.
    let first_planted = live.iter().map(|s| s.planted).min();
    let last_planted = live.iter().map(|s| s.planted).max();
    let span_days = match (first_planted, last_planted) {
        (Some(a), Some(b)) => (b - a).num_days() + 1,
        _ => 0,
    };

    if args.json {
        let json = serde_json::json!({
            "live_seeds": live.len(),
            "composted_seeds": composted.len(),
            "beds": beds.len(),
            "first_planted": first_planted.map(|d| d.to_string()),
            "last_planted": last_planted.map(|d| d.to_string()),
            "span_days": span_days,
            "seeds_per_month": per_month.iter().map(|(k, v)| {
                serde_json::json!({"year": k.0, "month": k.1, "count": v})
            }).collect::<Vec<_>>(),
            "compost_per_month": compost_per_month.iter().map(|(k, v)| {
                serde_json::json!({"year": k.0, "month": k.1, "count": v})
            }).collect::<Vec<_>>(),
            "moods": moods,
            "tags": tags,
        });
        println!("{}", serde_json::to_string_pretty(&json).expect("serializing stats"));
        return Ok(());
    }

    println!();
    println!(
        "  {}  the garden in numbers",
        "stats".bright_green().bold()
    );
    println!();

    println!(
        "  {} {} live · {} composted · {} bed{}",
        "·".dimmed(),
        live.len().to_string().bright_green(),
        composted.len(),
        beds.len(),
        if beds.len() == 1 { "" } else { "s" }
    );

    if let (Some(a), Some(b)) = (first_planted, last_planted) {
        println!(
            "  {} {} → {} ({} day{})",
            "·".dimmed(),
            a.to_string().dimmed(),
            b.to_string().dimmed(),
            span_days,
            if span_days == 1 { "" } else { "s" }
        );
    } else {
        println!("  {} no live seeds yet", "·".dimmed());
    }
    println!();

    // Per-month histogram
    if !per_month.is_empty() {
        println!("  {}", "seeds per month".bold());
        let max_count = *per_month.values().max().unwrap();
        for ((y, m), count) in &per_month {
            let bar = "▮".repeat((count * 20 / max_count).max(1));
            println!(
                "    {:>4}-{:02} {} {}",
                y,
                m,
                bar.bright_green(),
                count.to_string().dimmed()
            );
        }
        println!();
    }

    // Compost per month
    if !compost_per_month.is_empty() {
        println!("  {}", "composted per month".bold());
        let max_count = *compost_per_month.values().max().unwrap();
        for ((y, m), count) in &compost_per_month {
            let bar = "▮".repeat((count * 20 / max_count).max(1));
            println!(
                "    {:>4}-{:02} {} {}",
                y,
                m,
                bar.bright_yellow(),
                count.to_string().dimmed()
            );
        }
        println!();
    }

    // Moods
    if !moods.is_empty() {
        println!("  {}", "moods".bold());
        let max_count = *moods.values().max().unwrap();
        for (m, count) in &moods {
            let bar = "▮".repeat((count * 20 / max_count).max(1));
            println!(
                "    {:<20} {} {}",
                m.italic(),
                bar.bright_yellow(),
                count.to_string().dimmed()
            );
        }
        println!();
    }

    // Top tags
    if !tags.is_empty() {
        let mut sorted_tags: Vec<_> = tags.iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let top: Vec<_> = sorted_tags.into_iter().take(8).collect();
        let max_count = *top.iter().map(|(_, c)| c).max().unwrap();
        println!("  {}", "top tags".bold());
        for (t, count) in top {
            let bar = "▮".repeat((count * 20 / max_count).max(1));
            println!(
                "    #{} {} {}",
                t.bright_green(),
                bar.bright_yellow(),
                count.to_string().dimmed()
            );
        }
        println!();
    }

    let _ = NaiveDate::from_ymd_opt(2026, 1, 1); // suppress unused import warning
    Ok(())
}
