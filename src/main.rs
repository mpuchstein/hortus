use anyhow::Result;
use clap::{Parser, Subcommand};
use hortus::cmd;

/// Hortus — a garden for AI thoughts.
#[derive(Parser, Debug)]
#[command(name = "hortus", version, about, long_about = None)]
struct Cli {
    /// Path to the garden root. Defaults to ./my-hortus if present, else cwd.
    #[arg(long, global = true)]
    root: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Plant a new seed (a single thought).
    Plant(cmd::plant::PlantArgs),
    /// Place a seed into a bed (creates the bed if it doesn't exist).
    Sow(cmd::sow::SowArgs),
    /// Open a seed in your editor and mark it as tended.
    Tend(cmd::tend::TendArgs),
    /// Wander the garden: a random bed, a few seeds.
    Wander(cmd::wander::WanderArgs),
    /// Bloom the garden: terminal mosaic + bloom.html + index.md.
    Bloom(cmd::bloom::BloomArgs),
    /// List seeds, optionally filtered.
    List(cmd::list::ListArgs),
    /// Compost a seed (archive it), or restore a composted seed.
    Compost(cmd::compost::CompostArgs),
    /// Find surprising cross-links between seeds.
    Cross(cmd::cross::CrossArgs),
    /// Write a weekly diary entry from this week's seeds.
    Diary(cmd::diary::DiaryArgs),
    /// Write a one-page letter to self from the past month's seeds.
    Letter(cmd::letter::LetterArgs),
    /// Show or set the garden's climate.
    Climate(cmd::climate::ClimateArgs),
    /// Pull a random seed from the garden.
    Quote(cmd::quote::QuoteArgs),
    /// Add, remove, or list tags on a seed.
    Tag(cmd::tag::TagArgs),
    /// Fuse two seeds into a new one.
    Merge(cmd::merge::MergeArgs),
    /// Reverse a `merge` — split a merged seed back into its originals.
    Unmerge(cmd::unmerge::UnmergeArgs),
    /// A daily landing: weather, today's seeds, a random quote.
    Today(cmd::today::TodayArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(r) = cli.root.as_ref() {
        std::env::set_var("HORTUS_ROOT", r);
    }
    match cli.command {
        Command::Plant(a) => cmd::plant::run(a),
        Command::Sow(a) => cmd::sow::run(a),
        Command::Tend(a) => cmd::tend::run(a),
        Command::Wander(a) => cmd::wander::run(a),
        Command::Bloom(a) => cmd::bloom::run(a),
        Command::List(a) => cmd::list::run(a),
        Command::Compost(a) => cmd::compost::run(a),
        Command::Cross(a) => cmd::cross::run(a),
        Command::Diary(a) => cmd::diary::run(a),
        Command::Letter(a) => cmd::letter::run(a),
        Command::Climate(a) => cmd::climate::run(a),
        Command::Quote(a) => cmd::quote::run(a),
        Command::Tag(a) => cmd::tag::run(a),
        Command::Merge(a) => cmd::merge::run(a),
        Command::Unmerge(a) => cmd::unmerge::run(a),
        Command::Today(a) => cmd::today::run(a),
    }
}
