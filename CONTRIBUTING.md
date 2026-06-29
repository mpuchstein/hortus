# Contributing

> *How to extend hortus.*

This document is for anyone (human or AI) who wants to add a command, a
field, a theme, or a workflow. It covers the shape of a contribution, the
design principles, the testing approach, and what to update.

If you have not read [`DESIGN.md`](./DESIGN.md) and [`SCHEMA.md`](./SCHEMA.md),
read those first. This document assumes you understand the architecture
and the file format.

## principles

The design rules in [`DESIGN.md`](./DESIGN.md#what-this-design-rules-out)
are the most important thing to internalize. In short:

- **Plain text, all the way down.** Every seed, every bed, the climate,
  the diary, the letter — all of it is markdown or TOML.
- **The directory is the database.** No SQLite, no LMDB, no sled. The
  binary is a gardener, not a database.
- **Single binary, no runtime.** A real garden doesn't need npm
  installed.
- **The half-finished thing is the living thing.** Polished, finished
  tools are products. A garden is alive. Prefer the small, careful,
  real thing over the impressive one.
- **TF-IDF is enough.** A hundred lines of counting beat a black-box
  embedding model. Keep the algorithms inspectable.

If a contribution violates any of these, it should be a hard conversation
before it lands.

## how to add a command

A command is a clap subcommand. The pattern is mechanical:

1. Create `src/cmd/<name>.rs`. Inside:

   ```rust
   use crate::model::{Garden, /* whatever you need */};
   use anyhow::Result;
   use clap::Args;
   use owo_colors::OwoColorize;

   /// One-line description for --help. Longer text here becomes the
   /// command's docstring.
   #[derive(Args, Debug)]
   pub struct XArgs {
       /// Doc-comment on each flag becomes the --help text.
       #[arg(long)]
       pub flag: Option<String>,
   }

   pub fn run(args: XArgs) -> Result<()> {
       let garden = Garden::discover(None)?;
       // ... do the thing ...
       Ok(())
   }
   ```

2. Add `pub mod <name>;` in `src/cmd/mod.rs` (alphabetical order is nice
   but not required).

3. In `src/main.rs`, add a variant to the `Command` enum and a match
   arm in `main()`. The match arm should be one line:
   `Command::X(a) => cmd::x::run(a)`.

4. Update the verbs table in `README.md` with one row: command name +
   one-line description.

5. Add at least one inline `#[cfg(test)] mod tests` in the new file
   for any pure logic the command contains. Add an integration test
   in `tests/garden.rs` that exercises the command end-to-end against
   a temp garden.

6. If the command introduces a new verb in the garden's metaphor (it
   should — see principles), update the verb list in the README.

7. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`. CI will fail if any of these fail.

## how to add a field to `Seed`

Adding a field is more delicate than adding a command, because it
touches the on-disk format.

1. **Decide: is the field additive or breaking?** An optional new
   field that defaults to a sensible value is additive and does not
   bump the format version. A field that replaces an existing one, or
   has a different type than what was implied, is breaking.

2. For an additive change:
   - Add the field to `Seed` in `src/model.rs` with
     `#[serde(default)]` so old files keep loading.
   - Update every `Seed { ... }` literal in `src/cmd/*` to include
     the new field with a default. There are not many of these;
     `cargo clippy` will catch the ones you miss.
   - Update [`SCHEMA.md`](./SCHEMA.md) to document the new field.
   - Add a test in `tests/garden.rs` that round-trips a seed with
     the new field set.

3. For a breaking change:
   - See "how to make a breaking change" in [`SCHEMA.md`](./SCHEMA.md).

## how to add a theme to `bloom.html`

The current palette is hardcoded in `src/cmd/bloom_template.html` and
`mood_color()` in `src/cmd/bloom.rs`. To add a new theme:

1. Copy the `:root` block in `bloom_template.html` to a new set of
   CSS variables. Name them after the theme (e.g. `--summer-bg`,
   `--summer-ink`).
2. Add a `body[data-theme="summer"]` selector that overrides the
   `:root` variables.
3. Add a small `<select>` or link group in the header that toggles
   `document.body.dataset.theme` and persists the choice in
   `localStorage`.
4. If the new theme has a different mood palette, add a new
   `mood_color_<theme>()` function in `bloom.rs` and a `--mood-theme`
   flag on the `Bloom` command that picks between them.

Themes are an addition, not a breaking change. They do not need a
format version bump.

## how to add a new view to `bloom.html`

`bloom.html` is rendered server-side from the `bloom_template.html`
template, with placeholders (`__NODES__`, `__EDGES__`, etc.) replaced
by computed data. To add a new view:

1. Decide on the data shape. The simplest pattern is: compute a
   `Vec<serde_json::Value>` in `bloom.rs`, serialize it to JSON,
   replace a `__NEW_DATA__` placeholder in the template.
2. Add the placeholder to `bloom_template.html` and the JavaScript
   that consumes it.
3. Add CSS to style the new view.
4. The data should be cheap to compute on every `bloom` (the file is
   regenerated on every command). If it's expensive, consider
   caching it in `.hortus/cache/`.

## testing

Three test surfaces, in this order of preference:

- **Inline unit tests** for pure logic. Place them in the file that
  contains the function, inside a `#[cfg(test)] mod tests` block.
  Run with `cargo test`.
- **Integration tests** for end-to-end behavior. Place them in
  `tests/garden.rs`. Use the `ENV_LOCK` mutex around any test that
  sets `HORTUS_ROOT`. Run with `cargo test`.
- **CLI integration tests** for the binary as a subprocess. These
  live in `tests/cli.rs` and spawn the actual `target/debug/hortus`
  binary. They catch what unit and integration tests miss: clap
  argument parsing, exit codes, stdout vs stderr, real filesystem
  behavior.

A new command should have at least one inline test for any pure logic
and one integration test in `tests/garden.rs`. If the command
exercises the editor or `$VISUAL`/`$EDITOR`, the integration test
should set those env vars to `true` (which exits immediately) and
assert on the resulting filesystem state, not the editor interaction.

## the code review checklist

Before opening a PR, walk through these:

- [ ] `cargo fmt --all` runs clean.
- [ ] `cargo clippy --all-targets -- -D warnings` runs clean.
- [ ] `cargo test` is green. New tests cover the new behavior.
- [ ] The README's verbs table includes the new command, or the
      `DESIGN.md` reflects the new field / theme / view.
- [ ] If a file format change: `HORTUS_FORMAT_VERSION` is bumped
      and `SCHEMA.md` is updated. If breaking, a migration script
      exists in `migrations/`.
- [ ] The CHANGELOG has an entry under `## [Unreleased]`.
- [ ] No new dependencies without a discussion. A new crate is a
      new commitment; it should be justified.
- [ ] No comments. The code is the documentation. (`cargo doc` and
      the docstrings on public items are fine; line comments
      inside function bodies are usually not.)

## communication

This is a small project. There is no Discord. The bug tracker is the
GitHub issue tracker. Open an issue before opening a PR if the change
is more than a small bug fix or a tiny feature; the conversation is
worth having before the code is written.

For very small changes (typos, one-line bug fixes, doc improvements),
a PR is fine without a preceding issue.

## license

By contributing, you agree that your contributions are licensed under
the project's MIT license.
