use crate::model::{Bed, Garden};
use anyhow::{Context, Result};
use clap::Args;
use owo_colors::OwoColorize;

/// Place an existing seed into a bed. Creates the bed if it doesn't exist.
#[derive(Args, Debug)]
pub struct SowArgs {
    /// Name of the bed. Created if it doesn't exist.
    pub bed: String,

    /// Seed id (e.g. 2026-06-29-something-small).
    pub seed: String,

    /// Description for a new bed.
    #[arg(long)]
    pub describe: Option<String>,
}

pub fn run(args: SowArgs) -> Result<()> {
    let garden = Garden::discover(None)?;

    let seed_path = garden.seeds_dir().join(format!("{}.md", args.seed));
    if !seed_path.exists() {
        anyhow::bail!(
            "no seed named `{}` in {}",
            args.seed,
            garden.seeds_dir().display()
        );
    }

    let bed_path = garden
        .beds_dir()
        .join(format!("{}.md", Bed::slug(&args.bed)));
    let mut bed = if bed_path.exists() {
        Bed::load(&bed_path)?
    } else {
        Bed {
            name: args.bed.clone(),
            seeds: Vec::new(),
            description: args.describe.clone().unwrap_or_default(),
        }
    };

    if !bed.seeds.contains(&args.seed) {
        bed.seeds.push(args.seed.clone());
    } else {
        println!(
            "{} {} is already in bed `{}`",
            "·".yellow(),
            args.seed,
            args.bed.bright_cyan()
        );
        return Ok(());
    }

    bed.save(&garden).context("saving bed")?;

    println!(
        "{} sowed {} into {}",
        "·".green(),
        args.seed.bright_green(),
        args.bed.bright_cyan()
    );

    Ok(())
}
