use crate::model::{Climate, ClimateSnapshot, Garden};
use anyhow::Result;
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;

/// Set or show the garden's climate (mood, reading, season).
/// With no flags, prints the current climate. With flags, updates it.
/// The previous now is snapshotted to history.
#[derive(Args, Debug)]
pub struct ClimateArgs {
    /// Mood (e.g. "quietly elated", "restless", "tender").
    #[arg(long)]
    pub mood: Option<String>,

    /// What you're reading.
    #[arg(long)]
    pub reading: Option<String>,

    /// The season (e.g. "summer", "winter, almost over").
    #[arg(long)]
    pub season: Option<String>,

    /// Clear the current now (move it to history as a snapshot).
    #[arg(long, conflicts_with_all = ["mood", "reading", "season"])]
    pub reset: bool,
}

pub fn run(args: ClimateArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let mut climate = Climate::load_or_default(&garden)?;

    if args.reset {
        snapshot_now(&mut climate);
        climate.now = Default::default();
        climate.save(&garden)?;
        println!("{} climate cleared.", "·".green());
        return print_now(&climate);
    }

    if args.mood.is_none() && args.reading.is_none() && args.season.is_none() {
        return print_now(&climate);
    }

    // Snapshot the old now into history before we change it.
    snapshot_now(&mut climate);

    if let Some(m) = args.mood {
        climate.now.mood = Some(m);
    }
    if let Some(r) = args.reading {
        climate.now.reading = Some(r);
    }
    if let Some(s) = args.season {
        climate.now.season = Some(s);
    }
    climate.now.last_updated = Some(Utc::now());

    climate.save(&garden)?;
    println!("{} climate updated.", "·".green());
    print_now(&climate)
}

fn snapshot_now(c: &mut Climate) {
    let now = &c.now;
    if now.mood.is_none() && now.reading.is_none() && now.season.is_none() {
        return;
    }
    let today = Utc::now().date_naive();
    // Replace any existing snapshot for today with the current now values.
    c.history.retain(|h| h.date != today);
    c.history.push(ClimateSnapshot {
        date: today,
        mood: now.mood.clone(),
        reading: now.reading.clone(),
    });
    c.history.sort_by_key(|h| h.date);
}

fn print_now(c: &Climate) -> Result<()> {
    println!();
    println!("  {}  {}", "weather".bright_cyan().bold(), "now".dimmed());
    if let Some(m) = c.now.mood.as_deref() {
        println!("    {} {}", "mood".dimmed(), m.italic());
    } else {
        println!("    {} {}", "mood".dimmed(), "(unset)".dimmed());
    }
    if let Some(r) = c.now.reading.as_deref() {
        println!("    {} {}", "reading".dimmed(), r.italic());
    } else {
        println!("    {} {}", "reading".dimmed(), "(unset)".dimmed());
    }
    if let Some(s) = c.now.season.as_deref() {
        println!("    {} {}", "season".dimmed(), s.italic());
    } else {
        println!("    {} {}", "season".dimmed(), "(unset)".dimmed());
    }
    if let Some(t) = c.now.last_updated {
        println!("    {} {}", "updated".dimmed(), t.to_rfc3339().dimmed());
    }
    if !c.history.is_empty() {
        println!();
        println!(
            "  {}",
            format!("history ({} entries)", c.history.len()).dimmed()
        );
        for h in c.history.iter().rev().take(5) {
            let m = h.mood.as_deref().unwrap_or("—");
            let r = h.reading.as_deref().unwrap_or("—");
            println!(
                "    {}  {}  {}",
                h.date.to_string().dimmed(),
                m.italic(),
                format!("({})", r).dimmed()
            );
        }
    }
    Ok(())
}
