# Roadmap to 1.0.0

> *What it would take to call this `1.0.0` for real.*

This document exists because the previous attempt to tag this project
`1.0.0` was premature. The number got stamped on a few hours of work with
no external use, no deprecation cycle, no empirical test of whether the
API holds up. That is not what `1.0.0` is for.

`1.0.0` is a number that means *we have used this ourselves and others
have used it with us, the API has survived contact with reality, and the
project is ready for general use*. The current state is none of those
things. This roadmap is the path to making them true.

The current version is **`0.2.0`**. The on-disk format is at version
`1`, documented in [`SCHEMA.md`](./SCHEMA.md), and will not change in
a breaking way without a major version bump.

## the shape of the path

```
0.2.0  feedback round          ← we are here
0.3.0  performance
0.4.0  features round one
0.5.0  first deprecation cycle
0.6.0  migration
0.7.0  features round two
0.8.0  second deprecation cycle
0.9.0  release candidate
1.0.0  the real one
```

Each step is a minor version. The major version stays `0` until
`1.0.0`. The on-disk format version stays `1` across all of them, with
additive changes landing in minor versions and breaking changes
requiring `2.0.0`.

## what each version is for

### 0.2.0 — *we are here*

The first post-feedback release. The shape: a stable file format
version, a schema spec, a contribution story, three new commands
(`forage`, `stats`, `untend`), `--json` output, and 50 tests across
three layers (unit, garden, CLI).

This release was tagged `1.0.0` in haste the same day it was cut. The
tag was deleted and the release re-tagged as `0.2.0`. The content
is unchanged; the number is the promise, and the promise was too big.

### 0.3.0 — performance

Real performance work. The current `cross` is O(n²) in the number of
seeds. The current `bloom` simulation is hardcoded to 600 ticks. The
`tag list` walks the entire garden. None of this is measured.

The shape:

- Benchmarks for `cross`, `bloom`, `tag list` on synthetic gardens of
  100, 1k, 10k seeds.
- A threshold-based prefilter for `cross` (drop pairs whose largest
  shared-word frequency exceeds a cutoff) to extend its useful range.
- An incremental bloom: only re-render the parts of `bloom.html` whose
  underlying seeds have changed.
- A real cache in `.hortus/cache/` for expensive computations, with
  invalidation on `plant`, `sow`, `compost`, etc.

The gate: 0.3.0 ships when the benchmark suite is committed and the
worst-case behavior is bounded.

### 0.4.0 — features round one

The first round of features that are clearly missing. Candidate list:

- **`backlinks <id>`** — find seeds that reference this one. Currently
  reachable only with `grep`. A structured view would close a real gap.
- **Themes for `bloom.html`** — at least two palettes (the current
  "moss" and a "summer" light theme) with a toggle that persists in
  `localStorage`. Documented in `CONTRIBUTING.md` as an extension point.
- **`prune <id>`** — split a long seed into multiple shorter seeds
  by paragraph. Hard to do algorithmically; the implementation will
  probably be a thin editor interface rather than an auto-splitter.
- **`import <path>`** — plant a markdown file (or a directory of
  markdown files) as seeds. The other half of the existing implicit
  export story.

The gate: 0.4.0 ships when at least three of the four are done and
have integration tests. The fourth may move to 0.5.0.

### 0.5.0 — first deprecation cycle

The first time something is marked as deprecated. The candidate for
deprecation: the existing `wander --stale` flag, which is being
superseded by a more general `untend` workflow. The deprecation
notice goes in `--help`, the README, and the CHANGELOG. The flag
keeps working but prints a warning.

The deprecation policy:

- A deprecated flag is marked in `--help` with `[DEPRECATED]`.
- It keeps working for at least one minor version after deprecation.
- It is removed at the next major version (or the next minor version
  after a deprecation grace period of 3 months, whichever is later).
- Migration is documented in `CHANGELOG.md` and `ROADMAP.md`.

The gate: 0.5.0 ships when at least one feature is in a deprecated
state with a clear migration path documented.

### 0.6.0 — migration

The first real migration. Whatever was deprecated in 0.5.0 is
removed. A migration tool (`migrations/0.5-to-0.6.sh` or similar)
transforms existing gardens. The format version stays `1`; the
migration is at the command-name / flag-name level, not the file
format.

The gate: 0.6.0 ships when the migration is written, tested on
fixtures, and documented.

### 0.7.0 — features round two

The second round of features, picked from what `0.4.0` did not
deliver and what feedback has surfaced since `0.3.0`. The candidate
list at this point is whatever the project needs then.

The gate: 0.7.0 ships when there are at least two new features with
tests.

### 0.8.0 — second deprecation cycle

Whatever is obsolete by now. Likely candidates: a flag that turned
out to be a mistake, a command that nobody used, a code path that
the migration at 0.6.0 left in.

The gate: 0.8.0 ships with at least one new deprecation.

### 0.9.0 — release candidate

A version with all deprecations applied, no new features, and a
CHANGELOG entry that explicitly invites testing. If the format
breaks after 0.9.0, the project goes back to 0.x.

The gate: 0.9.0 ships when no feature work has happened for at
least two weeks and all open issues are either closed or explicitly
deferred to a post-1.0 release.

### 1.0.0 — the real one

The first version that ships after real use. Concretely:

- At least 90 days between 0.9.0 and 1.0.0.
- The format has been stable across at least three minor versions
  (e.g., 0.5, 0.6, 0.7, 0.8, 0.9).
- A migration has been written and tested, then a second migration
  has been written and tested.
- At least one person other than the author has used the tool for
  a non-trivial task and reported back.
- The CHANGELOG's `[Unreleased]` section is empty.
- The format version is still `1`.

The 1.0.0 release is the moment of "this is done enough to commit
to." It is not the moment of "this is feature complete." The
project will continue to grow after 1.0.0; the difference is that
after 1.0.0, growth cannot break the format or the CLI surface
without a major version bump.

## what this roadmap rules out

- **A fixed timeline.** Real roadmaps are pushed by feedback. The
  gates above are the conditions for each version, not the dates.
- **A feature checklist.** The candidate lists are starting points.
  The actual content of 0.4, 0.7, and beyond will be shaped by what
  users want.
- **A 1.0.0 in 2026.** The first version of this roadmap is honest
  about its own uncertainty. 1.0.0 will be tagged when the gates
  pass, not before.

## what this roadmap commits to

- The format version `1` will not change in a breaking way until
  `2.0.0`. Additive changes are fine across all versions.
- The CLI surface (command names, flag names, default values) will
  not change in a breaking way until `1.0.0`, and even then the
  standard deprecation grace period applies.
- Each minor version will ship with a CHANGELOG entry that says
  what changed, what was deprecated, and what the migration path is
  if anything was removed.
- The CI badge at the top of the README will be green at every
  release.

## how to influence this roadmap

Open an issue. The `0.4.0` candidate list is a starting point, not a
contract. If you want a feature in `0.4.0`, file an issue. If you
disagree with a deprecation in `0.5.0`, file an issue. If you have
used the tool and have feedback, the `0.3.0` benchmarks should
include a "user-visible regressions" section, and your feedback is
exactly the kind of thing that goes there.

The roadmap is a path, not a fence.
