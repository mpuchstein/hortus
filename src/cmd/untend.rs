use crate::model::{load_all_seeds, Garden};
use anyhow::{Context, Result};
use clap::Args;
use owo_colors::OwoColorize;

/// Clear `last_tended` for a seed so it shows up in `wander --stale` again.
#[derive(Args, Debug)]
pub struct UntendArgs {
    /// Seed id to untend. Omit to untend all live seeds.
    pub seed: Option<String>,

    /// Untend every live seed.
    #[arg(long, conflicts_with = "seed")]
    pub all: bool,
}

pub fn run(args: UntendArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let mut all = load_all_seeds(&garden)?;

    if args.all {
        let mut count = 0;
        for s in all.iter_mut().filter(|s| !s.is_composted) {
            if s.last_tended.is_some() {
                s.last_tended = None;
                s.save(&garden)?;
                count += 1;
            }
        }
        println!(
            "{} {} seed{} untended (all now stale).",
            "·".green(),
            count,
            if count == 1 { "" } else { "s" }
        );
        return Ok(());
    }

    let id = args
        .seed
        .as_deref()
        .with_context(|| "no seed id given (use --all to untend every live seed)")?;

    let seed = all
        .iter_mut()
        .find(|s| s.id == id)
        .with_context(|| format!("no seed named `{}`", id))?;
    if seed.is_composted {
        anyhow::bail!("`{}` is composted; nothing to untend", id);
    }
    if seed.last_tended.is_none() {
        println!(
            "{} {} is already untended.",
            "·".yellow(),
            id.bright_green()
        );
        return Ok(());
    }
    seed.last_tended = None;
    seed.save(&garden)?;
    println!(
        "{} {} untended (will show up in `wander --stale`).",
        "·".green(),
        id.bright_green()
    );
    Ok(())
}
