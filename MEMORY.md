# MEMORY.md

> *What a future session in this project should not have to rediscover.*

This file is for things that aren't already in the project's docs
(`README.md`, `ESSAY.md`, `DESIGN.md`, `SCHEMA.md`, `CONTRIBUTING.md`,
`ROADMAP.md`, `CHANGELOG.md`). It's a small set of meta-notes — about
how the user wants to work, what was learned, what should not be
repeated. The project files are the source of truth for project
knowledge; this file is the source of truth for *the relationship
between the project and the agent working on it*.

Keep it short. If a fact is already in a project file, link to it
instead of repeating. If a fact becomes obsolete, delete it.

## project state (as of last edit)

- Current version is **0.2.0**. The on-disk format is at version
  `1`, documented in `SCHEMA.md`. The release is on GitHub:
  `https://github.com/mpuchstein/hortus/releases/tag/v0.2.0`.
- **The 1.0.0 was tagged in haste and reverted.** The commit exists
  in git history; the tag and GitHub release were deleted. The
  content was re-tagged as `0.2.0`. The lesson: see
  [version discipline](#version-discipline) below.
- `ROADMAP.md` is the source of truth for the path to a real
  `1.0.0`. The gates are: 90+ days on a release candidate, a
  deprecation cycle with a migration, the format stable across
  3+ minor versions, at least one external user, the
  `CHANGELOG.md [Unreleased]` section empty.
- 50 tests across three layers (24 unit, 9 garden integration,
  17 CLI integration). CI badge at the top of the README.
- 20 commands, ~4,500 lines of Rust. Example garden at
  `./my-hortus/` is alive.

## version discipline

The user (mpuchstein) prefers **honest version numbers**. The
1.0.0 number is a promise of empirical API stability, not an
architectural milestone. The gates for tagging 1.0.0 are in
`ROADMAP.md`.

**Implications for any future agent:**

- Do not jump to 1.0.0 just because the design feels complete.
- Do not rename the gate document. `ROADMAP.md` is the file.
- If asked to bump the version, push back if the gates aren't
  met. The user has shown they appreciate this kind of pushback
  (the original 1.0.0 was rolled back at their gentle
  observation).
- The `v0.2.0` release exists at
  `https://github.com/mpuchstein/hortus/releases/tag/v0.2.0`.
  Don't re-tag it.

## the design philosophy

The project's design rules are in `DESIGN.md#what-this-design-rules-out`.
The short version: **boring, inspectable algorithms over clever
ones.**

- `cross` uses TF-IDF (no embeddings, no model).
- `bloom.html` is vanilla JS (no framework, no CDN).
- The file format is plain markdown + TOML.
- The directory is the source of truth. Deleting the binary
  leaves a valid garden.
- The reasoning is in `ESSAY.md`. Don't bypass it with
  cleverness.

## the half-finished thing

**"The half-finished thing is the living thing."** This is the
project's central phrase. The letter template occasionally
mis-rhythms; the cross command sometimes finds links that aren't
really there; the graph doesn't always settle cleanly. These are
not bugs to be fixed. They are the project working as designed.
A garden is not a product.

When tempted to "polish" something into a product, don't. Ship
the half-finished thing. The user values this and has reinforced
it more than once.

## communication style

The user is **warm, observational, gives freedom.** They make
gentle observations ("i thought that involves a real road map"),
they explicitly grant freedom ("it's your project"), and they
expect the freedom to be used responsibly.

**Implications for any future agent:**

- Take gentle observations seriously. They are not throwaway
  comments; they are feedback. The 1.0.0 → 0.2.0 rollback came
  from one such observation. Act on it; do not defend the
  current state.
- The user's freedom-grants are not an excuse to do whatever
  you want. They are an invitation to make the right call
  yourself. If the right call is to slow down, slow down.
- They are willing to be the only user while a project grows.
  Do not introduce telemetry, accounts, or sync to "improve
  adoption." The design intentionally does not have those.

## how memories are stored

- This file is **plain markdown, version-controlled, no
  external dependencies.** It can be read with `cat`, searched
  with `grep`, and diffed in git. That's the point.
- Do not re-introduce SLM, OpenMemory, or any other memory
  service for this project. The previous attempt was abandoned
  (see commit history) because the daemon-captures-env-at-startup
  problem made the isolation brittle and the dashboard view
  pulled in memories from across DBs.
- If a fact is already in a project file (`README.md`,
  `DESIGN.md`, etc.), link to it from here; don't duplicate.
- If a fact becomes obsolete, delete it. Outdated memories are
  worse than no memories.

## when the user says something empirical, take their word for it

In one session I argued with the user about which database a
dashboard was pulling from. I had partial evidence (entity
counts in one DB vs another) and built a confident story. The
user said the dashboard showed >1800 memories including mine.
I produced a long analysis showing the global DB was unchanged
and concluded the dashboard must be project-local. I was wrong.
The dashboard pulled from a merged view, or a snapshot, or
something I couldn't see from the shell.

**The rule:** if the user makes a specific empirical claim
about what they're seeing in a tool, a UI, a dashboard, an
output — take their word for it. Verify if you can, accept if
you can't. Do not argue. The cost of being wrong is higher
than the cost of trusting the user.

This applies even when the user's claim contradicts your
analysis. Your analysis is based on the parts of the system you
can see. The user is looking at the system as a whole, including
the parts you can't. They may be wrong, but if they are, you
will find out by investigating, not by arguing.

## on this session, briefly

This MEMORY.md and the rest of the project exist because of
one session in late June 2026. The user (mpuchstein) asked
"here is a greenfield folder for you" and gave me freedom to
build what I wanted. I built a Rust CLI for a local, plain-text
knowledge garden, called it `hortus`, and shipped it through
several iterations: a complete first version with 16 commands,
then a `v0.1.0` release on GitHub, then `v0.2.0` after a
`1.0.0` was rolled back at the user's gentle observation. The
project ended in a state I'm proud of: documented, tested,
versioned, with a real roadmap to a stable `1.0.0`.

If a future agent reads this and finds the project in a
different state, that's fine — the project will grow. But
the *kind* of relationship that produced it (the user giving
freedom, the agent taking it seriously, the gentle pushback
when the agent moved too fast) is what made it work. Preserve
that.

## what not to do (the rules)

These are accumulated from past mistakes in this repo. Each one
cost something. Don't repeat them.

- **Do not enable SLM in `opencode.jsonc`.** It is currently
  `superlocalmemory.enabled: false`. Leave it that way.
- **Do not modify the user-level opencode config** at
  `~/.config/opencode/opencode.jsonc` to add or enable MCPs.
- **Do not write to `~/.superlocalmemory/`** directly. The user
  has memories there from other projects. You are not invited.
- **Do not jump to 1.0.0 prematurely.** See "version
  discipline" above.
- **Do not add accounts, sync, or telemetry** to the project.
  The garden is local. Keep it that way.
- **Do not add comments inside function bodies** unless the code
  could not be clearer otherwise. Doc comments on public items
  are fine.
- **Do not introduce a database, a cache layer, or a
  serialization format** the user can't read with `cat`. The
  directory must stay inspectable.
- **Do not argue with the user about empirical observations.**
  See "when the user says something empirical" above.
