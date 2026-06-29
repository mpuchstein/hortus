use crate::model::{load_all_seeds, Garden};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use owo_colors::OwoColorize;
use std::collections::BTreeMap;

/// Add, remove, or list tags.
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
    /// List the tags on a seed, or all tags in the garden (no id).
    List {
        /// Seed id. Omit to list every tag in the garden with its count.
        id: Option<String>,
    },
}

pub fn run(args: TagArgs) -> Result<()> {
    let garden = Garden::discover(None)?;

    match &args.action {
        TagAction::List { id: None } => return list_all(&garden),
        TagAction::List { id: Some(id) } => return list_one(&garden, id),
        _ => {}
    }

    let mut all = load_all_seeds(&garden)?;
    let target = all
        .iter()
        .position(|s| match &args.action {
            TagAction::Add { id, .. } => &s.id == id,
            TagAction::Remove { id, .. } => &s.id == id,
            TagAction::List { .. } => unreachable!(),
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
        TagAction::List { .. } => unreachable!(),
    }
    Ok(())
}

fn list_one(garden: &Garden, id: &str) -> Result<()> {
    let all = load_all_seeds(garden)?;
    let seed = all
        .iter()
        .find(|s| s.id == *id)
        .with_context(|| format!("no seed named `{}`", id))?;
    if seed.tags.is_empty() {
        println!("{} {} has no tags.", "·".dimmed(), seed.id);
    } else {
        let chips: Vec<String> = seed.tags.iter().map(|t| format!("#{}", t)).collect();
        println!(
            "{} {} — {}",
            "·".dimmed(),
            seed.id,
            chips.join(" ").bright_green()
        );
    }
    Ok(())
}

fn list_all(garden: &Garden) -> Result<()> {
    let seeds = crate::model::load_live_seeds(garden)?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for s in &seeds {
        for t in &s.tags {
            *counts.entry(t.clone()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        println!("{}", "no tags yet in the living garden.".dimmed());
        return Ok(());
    }
    println!(
        "{} {} tag{} across {} seed{}",
        "·".green(),
        counts.len(),
        if counts.len() == 1 { "" } else { "s" },
        seeds.len(),
        if seeds.len() == 1 { "" } else { "s" },
    );
    let max_count = *counts.values().max().unwrap();
    for (tag, count) in &counts {
        let bar_len = (count * 20 / max_count.max(1)).max(1);
        let bar = "▮".repeat(bar_len);
        println!(
            "    {} {} {}",
            format!("#{}", tag).bright_green(),
            bar.bright_yellow(),
            format!("{}", count).dimmed()
        );
    }
    Ok(())
}
