# AGENTS.md

> *Project-level agent instructions. Read this before doing anything.*

This file is the project's way of speaking to future agents (human or
AI) that work in this directory. It does not affect the code; it
affects how agents should behave.

## what this project is

`hortus` is a Rust CLI for maintaining a local, plain-text, file-based
"garden" of thoughts. The directory is the source of truth. The
binary is a gardener.

The README, `ESSAY.md`, `DESIGN.md`, `SCHEMA.md`, `CONTRIBUTING.md`,
`ROADMAP.md`, and `CHANGELOG.md` describe the project. The current
version is `0.2.0`; the on-disk format is at version `1`. The path
to a real `1.0.0` is in `ROADMAP.md`.

## memory — read this

This project keeps agent memories in plain markdown, in
[`MEMORY.md`](./MEMORY.md). The directory is the source of truth;
the binary is a gardener; the same principle applies to what
agents remember.

**Why not SLM?** This project briefly used the superlocalmemory
(SLM) MCP for memories, scoped to `./.slm/`. It was abandoned
because:

- The CLI daemon captures `SL_MEMORY_PATH` at startup; a stale
  global daemon hijacks CLI calls and writes to
  `~/.superlocalmemory/`.
- The opencode config only protects the MCP server, not the CLI.
- The SLM dashboard picks up memories from across DBs and
  snapshots, which made the project-local memories visible in a
  view the user expected to be global.

**The rule for this project:** agent memories live in
`MEMORY.md`. Add to it when you learn something a future session
shouldn't have to rediscover. Read it before doing anything that
might be affected by past decisions.

**What not to do:**

- Do not enable SLM in `opencode.jsonc`. The current config has
  `superlocalmemory.enabled: false`. Leave it that way.
- Do not write to `~/.superlocalmemory/`. The user has memories
  there from other projects. You are not invited.
- Do not modify the user-level opencode config at
  `~/.config/opencode/opencode.jsonc` to add or enable MCPs.
  If you need a different tool, ask the user.

## other guidance for agents

- **Plain text is the source of truth.** Do not introduce a database,
  a cache layer, or a serialization format that the user can't read
  with `cat`. The directory should be inspectable.
- **The format is versioned.** If you are tempted to change a
  filename pattern, a frontmatter field, or a directory layout, read
  `SCHEMA.md` first. Breaking changes to the format require a major
  version bump and a migration script.
- **Tests are the contract.** Before adding a command, write at
  least one inline unit test for the pure logic and one integration
  test in `tests/garden.rs` or `tests/cli.rs`. The CI badge must
  stay green.
- **The roadmap is not a fence.** `ROADMAP.md` describes the path
  from `0.2.0` to a real `1.0.0`. If you have feedback, the place
  to put it is in a discussion with the user, not in a silent change
  of direction.
- **Comments are rare.** The code is the documentation. Doc
  comments on public items are fine; line comments inside function
  bodies are usually not. If you find yourself wanting to add a
  long line comment, ask whether the code itself could be clearer
  instead.
