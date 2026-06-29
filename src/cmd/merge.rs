use crate::model::{load_all_seeds, make_seed_id, unique_seed_id, Garden, Seed};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;

/// Fuse two seeds into a new one. The originals are composted with
/// epitaphs referencing the merge. The new body keeps both originals
/// in clearly marked sections, so the merge is in principle reversible.
#[derive(Args, Debug)]
pub struct MergeArgs {
    /// First seed id.
    pub a: String,
    /// Second seed id.
    pub b: String,

    /// Explicit id for the new seed. Otherwise auto-generated.
    #[arg(long)]
    pub into: Option<String>,

    /// Don't compost the originals (leave them in seeds/).
    #[arg(long)]
    pub keep_originals: bool,
}

pub fn run(args: MergeArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let all = load_all_seeds(&garden)?;

    let a = all
        .iter()
        .find(|s| s.id == args.a)
        .with_context(|| format!("no seed named `{}`", args.a))?;
    let b = all
        .iter()
        .find(|s| s.id == args.b)
        .with_context(|| format!("no seed named `{}`", args.b))?;

    if a.id == b.id {
        anyhow::bail!("cannot merge a seed with itself");
    }

    // Determine new id.
    let today = Utc::now().date_naive();
    let new_id = if let Some(custom) = args.into.as_deref() {
        custom.to_string()
    } else {
        let combined = format!("{} {}", first_line(a), first_line(b));
        let base = make_seed_id(today, &combined);
        unique_seed_id(&garden, &base)
    };
    if garden.seeds_dir().join(format!("{}.md", new_id)).exists()
        || garden.compost_dir().join(format!("{}.md", new_id)).exists()
    {
        anyhow::bail!("a seed named `{}` already exists", new_id);
    }

    // Merge tags.
    let mut tags: Vec<String> = a.tags.iter().chain(b.tags.iter()).cloned().collect();
    tags.sort();
    tags.dedup();

    // Mood: prefer the most recent planted.
    let mood = if a.planted >= b.planted {
        a.mood.clone()
    } else {
        b.mood.clone()
    };

    // Build the body.
    let body = format!(
        "> merged from `{}` ({}) and `{}` ({}).\n\n\
         ## from {}\n\n\
         {}\n\n\
         ## from {}\n\n\
         {}\n",
        a.id,
        a.planted,
        b.id,
        b.planted,
        a.id,
        a.body.trim(),
        b.id,
        b.body.trim()
    );

    let new_seed = Seed {
        id: new_id.clone(),
        planted: today,
        last_tended: Some(today),
        mood,
        tags,
        composted_at: None,
        epitaph: None,
        body,
        is_composted: false,
    };
    new_seed.save(&garden)?;

    println!(
        "{} merged {} and {} into {}",
        "·".green(),
        a.id.bright_green(),
        b.id.bright_green(),
        new_id.bright_cyan().bold()
    );

    // Compost the originals (unless told not to).
    if !args.keep_originals {
        for s in [&a, &b] {
            let mut s = (*s).clone();
            s.is_composted = true;
            s.composted_at = Some(today);
            s.epitaph = Some(format!("merged into `{}`", new_id));
            s.save(&garden)?;
        }
        println!(
            "{} {} and {} composted (revertible with `hortus compost --restore`).",
            "·".dimmed(),
            a.id,
            b.id
        );
    }

    Ok(())
}

fn first_line(s: &Seed) -> String {
    s.body
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .chars()
        .take(40)
        .collect()
}
