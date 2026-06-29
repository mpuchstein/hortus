# Changelog

All notable changes to hortus will be documented in this file. The format
is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project does its best to adhere to [Semantic Versioning](https://semver.org/).

## [0.1.0] — 2026-06-29

The first release of hortus as a complete, documented, tested tool. The
garden has a front gate, sixteen verbs, an essay behind it, and a CI
workflow that keeps it honest.

### Added

**The verbs** — sixteen commands, each a small piece of the gardening
metaphor:

- `plant` — quick capture, or open the editor with a template that
  already contains the climate and the last few seeds as context
- `sow` — place a seed in a bed (creates the bed if absent)
- `tend` — open a seed in `$VISUAL` / `$EDITOR` and mark it as watered
- `list` — filtered listing (by bed, tag, mood, since-date)
- `wander` — a random bed and a few seeds, or the oldest untended with
  `--stale`
- `quote` — a random seed, like a flower picked in passing
- `compost` — archive a seed with an epitaph, or restore a composted seed
- `cross` — find surprising cross-links between seeds via TF-IDF rare-word
  sharing; can write the result as a new bed
- `merge` — fuse two seeds into a new one with both bodies preserved in
  marked sections; originals are composted with epitaphs
- `unmerge` — reverse a `merge`, splitting a merged seed back into its
  originals
- `tag` — add, remove, or list tags (one seed, or all tags in the
  garden with counts)
- `diary` — write a weekly journal entry from this week's seeds
- `letter` — write a one-page letter to self from the past month's seeds,
  with mood-aware opening and closing
- `climate` — show or set the garden's weather (mood, reading, season);
  snapshots the old now to history
- `today` — a daily landing: weather, today's seeds, garden stats, a quote
- `bloom` — terminal mosaic + `bloom.html` (force-directed graph +
  mood timeline) + `index.md`

**The artifacts**:

- `bloom.html` — a self-contained file with a force-directed graph in
  vanilla JavaScript, no CDN, no build step. Includes a mood timeline at
  the top.
- `index.md` — a hand-written-feeling overview of the garden, regenerated
  on every `bloom`.
- `letters/<date>-letter.md` — written by `hortus letter`.
- `diary/<date>-diary.md` — written by `hortus diary`.

**The data model**:

- Plain markdown seeds with YAML frontmatter
- Plain markdown beds with YAML frontmatter
- TOML climate file with a `[now]` section and a `[[history]]` array
- One file per thought — `git diff` on a seed shows you exactly how the
  thought changed over time

**The tests**:

- 24 unit tests inline in `model.rs`, `text.rs`, `cmd/letter.rs`,
  `cmd/bloom.rs`, `cmd/list.rs`, `cmd/unmerge.rs`
- 9 integration tests in `tests/garden.rs` that exercise the high-level
  commands end-to-end against a fresh temp garden
- A `Mutex<()>` in the test suite serializes access to the `HORTUS_ROOT`
  env var so the integration tests pass deterministically in parallel

**The CI**:

- `.github/workflows/ci.yml` runs on every push and PR
- Steps: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo build --release`
- The badge at the top of the README reflects the latest `main` commit

**The example garden**:

- Ships populated in `./my-hortus/` so a fresh clone has a living thing
  to look at on day one
- 5 live seeds, 1 composted, 4 beds, a generated letter, a generated
  diary, a climate with a small history of moods

**The documents**:

- `README.md` — the front gate
- `ESSAY.md` — a personal note from the gardener
- `DESIGN.md` — the architecture, the file format, the algorithms, the
  extension points
- `LICENSE` — MIT
- `CHANGELOG.md` — this file

### Notes

This is `0.1.0`, not `1.0.0`. The garden is alive but the API is not yet
frozen. Future versions may rename commands, change the file format in
non-additive ways, or restructure the bloom template. The README, the
essay, and the design doc reflect the current shape.

The half-finished thing is the living thing. The garden is the living
thing. `0.1.0` is the moment the door is open.
