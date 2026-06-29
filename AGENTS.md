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

## memory isolation — read this

This project uses an opencode config (`opencode.jsonc`) that scopes
the **superlocalmemory (SLM)** MCP server to a project-local path
(`./.slm/`, which is `.gitignored`).

**The intent:** memories that the user (or any agent) collects while
working in this directory should stay in this directory. They should
not appear in the user's other projects, and memories from the user's
other projects should not appear here.

**The mechanism:** the `opencode.jsonc` sets `SL_MEMORY_PATH` to
`./.slm` in the MCP server's environment. The SLM server reads and
writes only to that path. The user's global SLM database at
`~/.superlocalmemory/` is not touched by agents working in this
directory, and is not read by them.

### what you should do as an agent

- **Use the SLM tools as normal.** They are scoped to this project.
  When you call `mcp__superlocalmemory__remember`, the memory is
  stored in `./.slm/`, not in the user's global database.
- **Do not modify the user-level opencode config** at
  `~/.config/opencode/opencode.jsonc` to enable additional MCPs.
  If you need a different tool, ask the user; do not silently change
  global state.
- **Do not change the `cwd` or `environment` fields of the SLM
  entry** in this project's `opencode.jsonc`. The point of those
  fields is to keep SLM local. Changing them would break the
  isolation guarantee.
- **Do not write to `~/.superlocalmemory/`** directly. The user has
  memories there from other projects. You are not invited.

### the daemon gotcha — read this too

The `opencode.jsonc` config sets `SL_MEMORY_PATH` for the MCP server
that opencode spawns. It does **not** affect the `slm` CLI. The CLI
talks to a long-running daemon, and the daemon captured its
`SL_MEMORY_PATH` at startup. This means:

- If the daemon was started with `SL_MEMORY_PATH=./.slm` (the
  intended state), the CLI is scoped to the project.
- If the daemon was started without it (e.g. inherited from a
  pre-config shell), the CLI hits `~/.superlocalmemory/` — the
  global DB the user does not want this project touching.

To make the CLI safe, **always run `slm` commands with the env
var explicitly set**, or use the wrapper at `./bin/slm-local`:

```bash
./bin/slm-local status
./bin/slm-local remember "..." --tags "..."
./bin/slm-local recall "..."
```

This sets `SL_MEMORY_PATH=./.slm` before invoking the CLI, so the
daemon is talking to the project DB regardless of how it was
started. If the daemon isn't running, `slm` auto-spawns one — and
that one will inherit the right env.

**After using `slm-local`, if you do `slm recall` without the
wrapper and the daemon was already running, you may hit the wrong
DB.** The wrapper is the only way to guarantee the right one.

If the daemon seems stuck on the global path, `slm restart` will
re-spawn it using the current shell's `SL_MEMORY_PATH`.

### what to do if the user wants to opt out

If the user wants to disable SLM entirely for this project (e.g.,
they don't want any MCP tools at all, or they want to use a different
memory tool), edit `opencode.jsonc` to set the SLM entry to:

```jsonc
"superlocalmemory": {
  "enabled": false
}
```

That is the only change required. The project will run with no SLM
tools at all.

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
