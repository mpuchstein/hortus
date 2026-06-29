use crate::model::{load_all_beds, load_live_seeds, Bed, Garden};
use crate::text::{document_frequencies, idf, tokenize};
use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;
use std::collections::HashSet;

/// Find surprising cross-links between seeds by shared rare words.
#[derive(Args, Debug)]
pub struct CrossArgs {
    /// How many surprising pairs to show.
    #[arg(long, default_value_t = 5)]
    pub top: usize,

    /// Also write the top pairs into a new bed with this name.
    #[arg(long)]
    pub write_bed: Option<String>,

    /// Description for the new bed (if --write-bed is set).
    #[arg(long)]
    pub describe: Option<String>,

    /// Minimum surprise score to include.
    #[arg(long, default_value_t = 0.0)]
    pub min_score: f32,

    /// Output as JSON instead of a human-readable list.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug)]
struct Pair {
    a: String,
    b: String,
    score: f32,
    shared: Vec<String>,
}

pub fn run(args: CrossArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = load_live_seeds(&garden)?;
    let beds = load_all_beds(&garden)?;

    if seeds.len() < 2 {
        println!(
            "{}",
            "the garden needs at least two live seeds to cross-pollinate.".yellow()
        );
        return Ok(());
    }

    // Tokenize each seed (body only — tags are shown as tags, not "surprise").
    let mut tokens_per_seed: Vec<Vec<String>> = Vec::with_capacity(seeds.len());
    for s in &seeds {
        tokens_per_seed.push(tokenize(&s.body));
    }

    // Build a set of (a, b) pairs that already share a bed; skip those.
    let mut already_linked: HashSet<(String, String)> = HashSet::new();
    for bed in &beds {
        for i in 0..bed.seeds.len() {
            for j in (i + 1)..bed.seeds.len() {
                let a = bed.seeds[i].clone();
                let b = bed.seeds[j].clone();
                let (a, b) = if a < b { (a, b) } else { (b, a) };
                already_linked.insert((a, b));
            }
        }
    }

    // Document frequencies.
    let df = document_frequencies(tokens_per_seed.iter().map(|v| v.as_slice()));
    let n = seeds.len();

    // For each pair, find shared rare words and score.
    let mut pairs: Vec<Pair> = Vec::new();
    for i in 0..seeds.len() {
        for j in (i + 1)..seeds.len() {
            let a_id = seeds[i].id.clone();
            let b_id = seeds[j].id.clone();
            let (key_a, key_b) = if a_id < b_id {
                (a_id.clone(), b_id.clone())
            } else {
                (b_id.clone(), a_id.clone())
            };
            if already_linked.contains(&(key_a.clone(), key_b.clone())) {
                continue;
            }
            // Find shared tokens.
            let set_i: HashSet<&String> = tokens_per_seed[i].iter().collect();
            let set_j: HashSet<&String> = tokens_per_seed[j].iter().collect();
            let shared: Vec<String> = set_i.intersection(&set_j).map(|s| (*s).clone()).collect();
            if shared.is_empty() {
                continue;
            }
            // Score = sum of idf for each shared word.
            let mut score = 0.0_f32;
            for w in &shared {
                let dfw = df.get(w).copied().unwrap_or(0);
                score += idf(dfw, n);
            }
            // Normalize by the size of the smaller set, so big seeds don't
            // dominate.
            let min_len = tokens_per_seed[i].len().min(tokens_per_seed[j].len()) as f32;
            if min_len > 0.0 {
                score /= min_len.sqrt();
            }
            if score < args.min_score {
                continue;
            }
            let mut shared_sorted = shared;
            shared_sorted.sort();
            pairs.push(Pair {
                a: a_id,
                b: b_id,
                score,
                shared: shared_sorted,
            });
        }
    }

    pairs.sort_by(|x, y| {
        y.score
            .partial_cmp(&x.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pairs.truncate(args.top);

    if pairs.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!(
                "{}",
                "no surprising cross-links found. the garden's thoughts have settled.".yellow()
            );
        }
        return Ok(());
    }

    if args.json {
        let json: Vec<serde_json::Value> = pairs
            .iter()
            .map(|p| {
                serde_json::json!({
                    "a": p.a,
                    "b": p.b,
                    "score": p.score,
                    "shared": p.shared,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("serializing pairs")
        );
        return Ok(());
    }

    println!(
        "{}",
        "cross-pollinations — shared rare words between seeds that don't share a bed:".bold()
    );
    println!();
    for p in &pairs {
        let shared_str = p
            .shared
            .iter()
            .map(|s| format!("{}", s.italic()))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {} {} {} {} {}",
            "✿".bright_green(),
            p.a.bright_green(),
            "×".dimmed(),
            p.b.bright_green(),
            format!("(score {:.2})", p.score).dimmed()
        );
        println!("      shared: {}", shared_str.dimmed());
    }

    // Optionally write as a new bed.
    if let Some(name) = args.write_bed {
        let bed = Bed {
            name: name.clone(),
            description: args.describe.clone().unwrap_or_else(|| {
                "Cross-pollinations — pairs of seeds that share rare words but not yet a bed."
                    .to_string()
            }),
            seeds: pairs
                .iter()
                .flat_map(|p| vec![p.a.clone(), p.b.clone()])
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
        };
        bed.save(&garden)?;
        println!();
        println!(
            "{} wrote a new bed `{}` ({} seeds, {} pairs).",
            "·".green(),
            name.bright_cyan(),
            bed.seeds.len(),
            pairs.len()
        );
    }

    Ok(())
}
