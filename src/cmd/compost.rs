use crate::model::{load_all_seeds, Garden};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;

/// Compost a seed (archive it) or restore a composted seed.
#[derive(Args, Debug)]
pub struct CompostArgs {
    /// Seed id to compost or restore.
    pub seed: String,

    /// A one-line epitaph to mark why this seed was let go.
    #[arg(long)]
    pub epitaph: Option<String>,

    /// Restore a composted seed back to the live garden.
    #[arg(long, conflicts_with = "epitaph")]
    pub restore: bool,
}

pub fn run(args: CompostArgs) -> Result<()> {
    let garden = Garden::discover(None)?;

    // Find the seed in either directory.
    let mut seed = load_all_seeds(&garden)?
        .into_iter()
        .find(|s| s.id == args.seed)
        .with_context(|| format!("no seed named `{}`", args.seed))?;

    if args.restore {
        if !seed.is_composted {
            println!(
                "{} {} is already in the living garden.",
                "·".yellow(),
                args.seed.bright_green()
            );
            return Ok(());
        }
        seed.is_composted = false;
        seed.composted_at = None;
        seed.epitaph = None;
        seed.save(&garden).context("restoring seed")?;
        println!(
            "{} {} returned to the living garden.",
            "·".green(),
            args.seed.bright_green()
        );
    } else {
        if seed.is_composted {
            println!(
                "{} {} is already composted.",
                "·".yellow(),
                args.seed.bright_green()
            );
            return Ok(());
        }
        seed.is_composted = true;
        seed.composted_at = Some(Utc::now().date_naive());
        if let Some(epi) = args.epitaph {
            seed.epitaph = Some(epi);
        }
        seed.save(&garden).context("composting seed")?;
        let epi_note = seed
            .epitaph
            .as_deref()
            .map(|e| format!("\n  {} {}", "epitaph:".dimmed(), e.italic()))
            .unwrap_or_default();
        println!(
            "{} {} composted.{}",
            "·".green(),
            args.seed.bright_green(),
            epi_note
        );
    }
    Ok(())
}
