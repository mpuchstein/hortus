use crate::cmd::quote::QuoteArgs;
use crate::model::{load_live_seeds, Garden};
use anyhow::Result;
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;

/// A daily landing for the garden. Shows the weather, today's seeds,
/// and a random flower in passing.
#[derive(Args, Debug)]
pub struct TodayArgs {
    /// Skip the random quote at the end.
    #[arg(long)]
    pub no_quote: bool,
    /// Output as JSON instead of a human-readable summary.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: TodayArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let today = Utc::now().date_naive();
    let climate = crate::model::Climate::load_or_default(&garden)?;
    let seeds = load_live_seeds(&garden)?;

    let today_seeds: Vec<_> = seeds.iter().filter(|s| s.planted == today).collect();
    let recently_tended: Vec<_> = seeds
        .iter()
        .filter(|s| s.last_tended == Some(today))
        .collect();

    if args.json {
        let json = serde_json::json!({
            "date": today.to_string(),
            "climate": {
                "mood": climate.now.mood,
                "reading": climate.now.reading,
                "season": climate.now.season,
            },
            "today_seeds": today_seeds.iter().map(|s| serde_json::json!({
                "id": s.id,
                "mood": s.mood,
                "first_line": s.body.lines().find(|l| !l.trim().is_empty()),
            })).collect::<Vec<_>>(),
            "recently_tended": recently_tended.iter()
                .filter(|s| !today_seeds.iter().any(|t| t.id == s.id))
                .map(|s| serde_json::json!({"id": s.id}))
                .collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("serializing today")
        );
        return Ok(());
    }

    println!();
    println!(
        "  {}  {}",
        "today".bright_green().bold(),
        today.to_string().dimmed()
    );
    println!();

    // Weather
    println!("  {}", "weather".bold());
    if let Some(m) = climate.now.mood.as_deref() {
        println!("    {} {}", "mood".dimmed(), m.italic());
    } else {
        println!("    {} {}", "mood".dimmed(), "(unset)".dimmed());
    }
    if let Some(r) = climate.now.reading.as_deref() {
        println!("    {} {}", "reading".dimmed(), r.italic());
    }
    if let Some(s) = climate.now.season.as_deref() {
        println!("    {} {}", "season".dimmed(), s.italic());
    }

    // Today
    println!();
    if today_seeds.is_empty() {
        println!("  {}", "today".bold());
        println!("    {} no seeds planted today", "·".dimmed());
    } else {
        println!(
            "  {}  {} seed{} planted",
            "today".bold(),
            today_seeds.len(),
            if today_seeds.len() == 1 { "" } else { "s" }
        );
        for s in &today_seeds {
            let mood = s
                .mood
                .as_deref()
                .map(|m| format!(" · {}", m))
                .unwrap_or_default();
            println!(
                "    {} {}{}",
                "·".dimmed(),
                s.id.bright_green(),
                mood.italic()
            );
        }
    }

    // Recently tended
    if !recently_tended.is_empty() {
        let newly_tended: Vec<_> = recently_tended
            .iter()
            .filter(|s| !today_seeds.iter().any(|t| t.id == s.id))
            .collect();
        if !newly_tended.is_empty() {
            println!();
            println!("  {}", "watered today".bold());
            for s in newly_tended {
                println!("    {} {}", "·".dimmed(), s.id.bright_green());
            }
        }
    }

    // Garden stats
    println!();
    let live = seeds.len();
    let composted = crate::model::load_all_seeds(&garden)?
        .iter()
        .filter(|s| s.is_composted)
        .count();
    let beds = crate::model::load_all_beds(&garden)?.len();
    println!(
        "  {}  {} live · {} composted · {} bed{}",
        "garden".bold(),
        live,
        composted,
        beds,
        if beds == 1 { "" } else { "s" }
    );

    // Quote
    if !args.no_quote {
        println!();
        let qargs = QuoteArgs {
            count: 1,
            bed: None,
        };
        crate::cmd::quote::run(qargs)?;
    }

    Ok(())
}
