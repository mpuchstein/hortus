use crate::model::{Garden, Seed};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Open a seed in your editor and mark it as tended.
#[derive(Args, Debug)]
pub struct TendArgs {
    /// Seed id to tend. If omitted, tend the most recently planted seed.
    pub seed: Option<String>,
}

pub fn run(args: TendArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = crate::model::load_all_seeds(&garden)?;

    if seeds.is_empty() {
        anyhow::bail!("the garden is empty. plant a seed first.");
    }

    let id = match args.seed {
        Some(id) => id,
        None => seeds.last().map(|s| s.id.clone()).unwrap(),
    };

    let path = garden.seeds_dir().join(format!("{}.md", id));
    if !path.exists() {
        anyhow::bail!("no seed named `{}` at {}", id, path.display());
    }

    let editor = editor_command();
    println!(
        "{} tending {} in {}",
        "·".green(),
        id.bright_green(),
        editor.first().map(String::as_str).unwrap_or("?")
    );
    let status = Command::new(editor.first().expect("editor resolved"))
        .args(&editor[1..])
        .arg(&path)
        .status()
        .context("launching editor")?;
    if !status.success() {
        anyhow::bail!("editor exited with {}", status);
    }

    // Re-read and update last_tended.
    let mut seed = Seed::load(&path)?;
    seed.last_tended = Some(Utc::now().date_naive());
    seed.save(&garden)?;

    println!("{} watered.", "·".green());
    Ok(())
}

fn editor_command() -> Vec<String> {
    let raw = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    simple_split(&raw)
}

/// Split a command line on whitespace, respecting simple double-quoted segments.
fn simple_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for c in s.chars() {
        match c {
            '"' => in_quote = !in_quote,
            c if c.is_whitespace() && !in_quote => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        vec![s.to_string()]
    } else {
        out
    }
}

#[allow(dead_code)]
pub fn write_to_temp(name: &str, body: &str) -> Result<PathBuf> {
    let mut f = tempfile::Builder::new()
        .prefix(&format!("hortus-{}", name))
        .suffix(".md")
        .tempfile()
        .context("creating temp file")?;
    f.write_all(body.as_bytes())?;
    Ok(f.path().to_path_buf())
}

#[allow(dead_code)]
pub fn read_file(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}
