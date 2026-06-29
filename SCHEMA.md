# Schema

> *The file format of a garden, and the rules for changing it.*

This document is the stability contract for the on-disk format of a
hortus garden. The current format version is **`1`**, declared in
`HORTUS_FORMAT_VERSION` in `src/model.rs`.

A garden is a directory. The directory is the source of truth. The
binary is a gardener that reads, manipulates, and writes the directory.
If you delete the binary, the directory is still a garden. If you have a
different binary, the directory is still a garden. This document is the
contract for what a directory needs to look like to be a garden that the
binary can read.

## version policy

The on-disk format version is a `u32` in `climate.toml`:

```toml
version = 1
```

- **Additive changes** (new optional fields in YAML frontmatter, new
  sections in `climate.toml`, new files in the directory) do not bump
  the version. A garden at version 1 is still a valid version-1 garden
  after an additive change.
- **Breaking changes** (renaming a field, changing a type, removing a
  field, changing the meaning of a directory) bump the version. A
  binary built for version 1 will refuse to read a version 2 garden
  with a clear error message.
- **Migration** is the user's job, not the binary's. A migration is a
  script that transforms a `version = N` garden into a `version = N+1`
  garden. Hortus will ship migration scripts (in `migrations/`) when a
  breaking change ships.

The current binary only knows how to read version 1. If it encounters
a garden with a higher version, it will print an error pointing to this
document and refuse to proceed.

## the directory

```
<root>/
├── seeds/         # YYYY-MM-DD-<slug>.md
├── compost/       # YYYY-MM-DD-<slug>.md
├── beds/          # <slug>.md
├── letters/       # YYYY-MM-DD-letter.md
├── diary/         # YYYY-MM-DD-diary.md
├── climate.toml   # [now] + [[history]] + version
├── index.md       # regenerated on `hortus bloom`
└── bloom.html     # regenerated on `hortus bloom`
```

`.hortus/` is a cache directory. Its contents are not part of the
format; the binary may use it for anything it wants and is free to
ignore it. Tools that read gardens should skip it.

## seeds

One markdown file per thought. The file is YAML frontmatter + body.

### filename

`YYYY-MM-DD-<slug>.md` where `<slug>` is a lowercase, dash-separated
version of the first few words of the body, with all non-alphanumeric
characters stripped. The slug is unique within `seeds/`. Slugs are
auto-disambiguated by appending `-2`, `-3`, etc. on collision.

### frontmatter

```yaml
id: 2026-06-29-a-rose-by-any-other-name
planted: 2026-06-29           # YYYY-MM-DD, the day the seed was planted
last_tended: 2026-06-29       # YYYY-MM-DD, optional, last time tend was run
mood: curious                 # optional free-form string
tags:                         # optional list of free-form strings
  - language
  - metaphor
composted_at: 2026-06-29      # YYYY-MM-DD, present iff the seed is in compost/
epitaph: this became the soil # optional, present iff the seed is in compost/
```

Field rules:

- `id`, `planted` are required.
- `last_tended`, `mood`, `tags`, `composted_at`, `epitaph` are optional.
- All fields are additive — new optional fields can be added in future
  versions without bumping the version.
- `is_composted` is *not* in the frontmatter. It is derived at load
  time from the file's parent directory. A seed file in `compost/` is
  composted; one in `seeds/` is live.

### body

The body is everything after the closing `---` of the frontmatter. It
is markdown. It may be empty. There is no length limit; seeds with
short bodies are encouraged.

## beds

One markdown file per bed, named `<slug>.md` where `<slug>` is the
bed's name lowercased with non-alphanumerics replaced by dashes.

### frontmatter

```yaml
name: on being an AI         # the bed's display name
seeds:                        # optional list of seed ids
  - 2026-06-26-on-rooms-without-windows
  - 2026-06-27-gardens-not-warehouses
description: Thoughts about... # optional, the body's text below frontmatter
```

### body

The body of a bed file is a free-form description, prose-only. It is
rendered as a paragraph in `bloom.html` and in the bed chip in
`index.md`.

## letters

Written by `hortus letter`. Filename: `letters/YYYY-MM-DD-letter.md`.
The body is the generated letter. The format is *not* part of the
schema — letters are ephemeral artifacts the binary produces, and the
tool does not read them back.

## diary entries

Written by `hortus diary`. Filename: `diary/YYYY-MM-DD-diary.md`. The
body is the generated diary. Same as letters: the format is not part
of the schema, and the tool does not read them back.

## climate.toml

The only file the binary both reads and writes that has a structured
schema.

```toml
version = 1

[now]
mood = "quietly elated"
reading = "Calvino, Invisible Cities"
season = "summer, almost over"
last_updated = 2026-06-29T15:00:00Z   # ISO 8601 UTC

[[history]]
date = "2026-06-26"
mood = "restless"
reading = "Ursula Le Guin, The Carrier Bag Theory of Fiction"
```

Fields:

- `version` (`u32`, required): the format version. See "version policy" above.
- `[now]`: the current weather. `mood`, `reading`, `season` are free-form
  strings; `last_updated` is ISO 8601 UTC. All four are optional.
- `[[history]]`: snapshots of past `now`s. Each snapshot has `date`
  (YYYY-MM-DD), and optional `mood` and `reading`. When the binary
  updates `[now]`, it appends the previous `[now]` to `[[history]]`.

Additive changes (new fields in `[now]`, new fields in `[[history]]`)
do not bump the version.

## bloom.html and index.md

Both are regenerated by `hortus bloom`. The HTML is a self-contained
file with embedded CSS and JavaScript; the JS template is
`src/cmd/bloom_template.html` and is part of the binary. The `index.md`
is plain markdown.

These are *outputs* of the binary, not inputs. A garden without them
is still a valid garden; the binary will regenerate them on the next
`bloom`. A reader that wants to read a garden does not need to parse
either of these files.

## what this schema rules out

- **Concurrent writes.** Two binaries writing the same seed at the same
  time will clobber. The directory is not transactional.
- **Symlinks, hardlinks, network filesystems.** The binary assumes
  ordinary files on a local filesystem. Behavior on a network share is
  untested.
- **Encrypted gardens.** The format is plain text. Encrypt the
  directory with whatever tool you like; the binary doesn't care.
- **File watchers.** The binary does not watch the directory for
  changes. If you edit a seed outside the binary, the binary will not
  notice until the next command that reads it.

## how to make a breaking change

If you are a contributor and you need to make a breaking change:

1. Open an issue. Describe the change and the migration path.
2. Bump `HORTUS_FORMAT_VERSION` in `src/model.rs`.
3. Write a migration script in `migrations/N-to-N+1.sh` (or `.py`,
   whatever fits). The script reads a `version = N` garden and writes
   a `version = N+1` garden, with `git`-friendly diffs.
4. Add a `CHANGELOG.md` entry under a new `## [Unreleased]` section
   describing the change, the migration, and the version bump.
5. Add an integration test that runs the migration on a fixture
   `version = N` garden and asserts the result is a valid
   `version = N+1` garden.

The binary will not perform the migration automatically. It will
detect the version mismatch and print a clear error pointing to the
migration script.
