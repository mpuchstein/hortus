use crate::model::{load_live_seeds, Garden, Seed};
use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;
use rand::seq::IndexedRandom;

/// Pull a random seed from the garden — a flower in passing.
#[derive(Args, Debug)]
pub struct QuoteArgs {
    /// Show multiple seeds.
    #[arg(long, default_value_t = 1)]
    pub count: usize,

    /// Only quote from seeds in this bed.
    #[arg(long)]
    pub bed: Option<String>,
}

pub fn run(args: QuoteArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = load_live_seeds(&garden)?;

    if seeds.is_empty() {
        println!("{}", "the garden is empty — nothing to quote yet.".yellow());
        return Ok(());
    }

    let pool: Vec<&Seed> = if let Some(name) = args.bed.as_deref() {
        let beds = crate::model::load_all_beds(&garden)?;
        let bed = beds
            .iter()
            .find(|b| crate::model::Bed::slug(&b.name) == crate::model::Bed::slug(name))
            .map(|b| b.seeds.clone())
            .unwrap_or_default();
        seeds
            .iter()
            .filter(|s| bed.contains(&s.id))
            .collect()
    } else {
        seeds.iter().collect()
    };

    if pool.is_empty() {
        println!("{}", "this bed is empty.".yellow());
        return Ok(());
    }

    let picks: Vec<&&Seed> = pool
        .choose_multiple(&mut rand::rng(), args.count.min(pool.len()))
        .collect();

    for (i, s) in picks.iter().enumerate() {
        if i > 0 {
            println!();
        }
        print_quote(s);
    }
    Ok(())
}

fn print_quote(s: &Seed) {
    let date = s.planted.format("%Y-%m-%d");
    println!(
        "  {} {}",
        date.dimmed(),
        s.id.bright_green()
    );
    for line in s.body.lines().take(6) {
        if line.trim().is_empty() {
            continue;
        }
        println!("    {}", line);
    }
    let lines = s.body.lines().filter(|l| !l.trim().is_empty()).count();
    if lines > 6 {
        println!("    {}", format!("… ({} more lines)", lines - 6).dimmed());
    }
    if let Some(m) = &s.mood {
        println!("    {} {}", "—".dimmed(), m.italic());
    }
}
