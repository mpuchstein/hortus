use crate::model::{load_all_beds, load_live_seeds, Bed, Garden, Seed};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;
use rand::seq::IndexedRandom;

/// Wander through the garden. Picks a random bed (or the whole garden) and
/// surfaces a few seeds to read.
#[derive(Args, Debug)]
pub struct WanderArgs {
    /// How many seeds to show.
    #[arg(long, default_value_t = 4)]
    pub count: usize,

    /// Wander a specific bed by name (slug) instead of picking one at random.
    #[arg(long)]
    pub bed: Option<String>,

    /// Surface seeds not tended in this many days, oldest first.
    #[arg(long)]
    pub stale: bool,

    /// How many days counts as stale (default 14).
    #[arg(long, default_value_t = 14)]
    pub stale_days: i64,
}

pub fn run(args: WanderArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = load_live_seeds(&garden)?;
    let beds = load_all_beds(&garden)?;

    if seeds.is_empty() {
        println!("{}", "the garden is empty.".yellow());
        println!("plant a seed:  hortus plant \"a small thought\"");
        return Ok(());
    }

    let climate = crate::model::Climate::load_or_default(&garden)?;

    if args.stale {
        return run_stale(&garden, &seeds, &args);
    }

    let pool: Vec<Seed> = if let Some(name) = args.bed.as_deref() {
        let bed = beds
            .iter()
            .find(|b| Bed::slug(&b.name) == Bed::slug(name))
            .with_context(|| format!("no bed named `{}`", name))?;
        bed.seeds
            .iter()
            .filter_map(|sid| seeds.iter().find(|s| &s.id == sid).cloned())
            .collect()
    } else if !beds.is_empty() {
        let bed = beds
            .choose(&mut rand::rng())
            .context("choosing a bed")?;
        println!(
            "{}  wandering into {}",
            "·".green(),
            bed.name.bright_cyan().bold()
        );
        if !bed.description.is_empty() {
            println!("  {}", bed.description.dimmed());
        }
        println!();
        bed.seeds
            .iter()
            .filter_map(|sid| seeds.iter().find(|s| &s.id == sid).cloned())
            .collect()
    } else {
        seeds.clone()
    };

    if pool.is_empty() {
        println!("{}", "this bed is empty. sow something into it.".yellow());
        return Ok(());
    }

    if let Some(m) = climate.now.mood.as_deref() {
        println!("  {} {}", "climate:".dimmed(), m.dimmed().italic());
    }
    if let Some(r) = climate.now.reading.as_deref() {
        println!("  {} {}", "reading:".dimmed(), r.dimmed().italic());
    }
    if climate.now.mood.is_some() || climate.now.reading.is_some() {
        println!();
    }

    let picks: Vec<&Seed> = pool
        .choose_multiple(&mut rand::rng(), args.count.min(pool.len()))
        .collect();

    for (i, s) in picks.iter().enumerate() {
        if i > 0 {
            println!();
        }
        print_seed(s);
    }

    Ok(())
}

fn run_stale(_garden: &Garden, seeds: &[Seed], args: &WanderArgs) -> Result<()> {
    let today = Utc::now().date_naive();
    let threshold = today - chrono::Duration::days(args.stale_days);

    let mut with_freshness: Vec<(&Seed, chrono::NaiveDate)> = seeds
        .iter()
        .map(|s| {
            let freshness = s.last_tended.unwrap_or(s.planted);
            (s, freshness)
        })
        .filter(|(_, f)| *f <= threshold)
        .collect();

    with_freshness.sort_by_key(|(_, f)| *f);

    if with_freshness.is_empty() {
        println!(
            "{}",
            format!(
                "no seeds older than {} days. the garden is fresh.",
                args.stale_days
            )
            .green()
        );
        return Ok(());
    }

    println!(
        "{}  stale — not tended in {} day{}",
        "·".yellow(),
        args.stale_days,
        if args.stale_days == 1 { "" } else { "s" }
    );
    println!();

    let picks: Vec<(&Seed, chrono::NaiveDate)> =
        with_freshness.into_iter().take(args.count).collect();

    for (i, (s, f)) in picks.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let age = (today - *f).num_days();
        println!(
            "  {} {} (last tended {}, {} day{} ago)",
            "◷".yellow(),
            s.id.bright_green(),
            f,
            age,
            if age == 1 { "" } else { "s" }
        );
        if let Some(line) = s.body.lines().find(|l| !l.trim().is_empty()) {
            println!("    {}", line.dimmed());
        }
    }
    Ok(())
}

fn print_seed(s: &Seed) {
    let date = s.planted.format("%Y-%m-%d");
    println!("  {} {}", date.dimmed(), s.id.bright_green());
    if let Some(m) = &s.mood {
        println!("    {} {}", "mood".dimmed(), m.italic());
    }
    if !s.tags.is_empty() {
        println!("    {} {}", "tags".dimmed(), s.tags.join(", ").dimmed());
    }
    for line in s.body.lines().take(6) {
        println!("    {}", line);
    }
    let lines = s.body.lines().count();
    if lines > 6 {
        println!("    {}", format!("… ({} more lines)", lines - 6).dimmed());
    }
}
