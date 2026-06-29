use crate::model::{load_all_seeds, Garden};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

/// Add, remove, or list tags on a seed.
#[derive(Args, Debug)]
pub struct TagArgs {
    #[command(subcommand)]
    pub action: TagAction,
}

#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// Add a tag to a seed.
    Add {
        /// Seed id.
        id: String,
        /// Tag to add.
        tag: String,
    },
    /// Remove a tag from a seed.
    Remove {
        /// Seed id.
        id: String,
        /// Tag to remove.
        tag: String,
    },
    /// List the tags on a seed.
    List {
        /// Seed id.
        id: String,
    },
}

pub fn run(args: TagArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let mut all = load_all_seeds(&garden)?;
    let target = all
        .iter()
        .position(|s| match &args.action {
            TagAction::Add { id, .. } => &s.id == id,
            TagAction::Remove { id, .. } => &s.id == id,
            TagAction::List { id } => &s.id == id,
        })
        .with_context(|| "no seed with that id")?;
    let seed = &mut all[target];

    match &args.action {
        TagAction::Add { tag, .. } => {
            if seed.tags.iter().any(|t| t == tag) {
                println!(
                    "{} {} already has tag `{}`",
                    "·".yellow(),
                    seed.id.bright_green(),
                    tag
                );
                return Ok(());
            }
            seed.tags.push(tag.clone());
            seed.tags.sort();
            seed.save(&garden)?;
            println!(
                "{} added `#{}` to {}",
                "·".green(),
                tag,
                seed.id.bright_green()
            );
        }
        TagAction::Remove { tag, .. } => {
            if let Some(pos) = seed.tags.iter().position(|t| t == tag) {
                seed.tags.remove(pos);
                seed.save(&garden)?;
                println!(
                    "{} removed `#{}` from {}",
                    "·".green(),
                    tag,
                    seed.id.bright_green()
                );
            } else {
                println!(
                    "{} {} has no tag `{}`",
                    "·".yellow(),
                    seed.id.bright_green(),
                    tag
                );
            }
        }
        TagAction::List { .. } => {
            if seed.tags.is_empty() {
                println!("{} {} has no tags.", "·".dimmed(), seed.id);
            } else {
                let chips: Vec<String> = seed
                    .tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect();
                println!(
                    "{} {} — {}",
                    "·".dimmed(),
                    seed.id,
                    chips.join(" ").bright_green()
                );
            }
        }
    }
    Ok(())
}
