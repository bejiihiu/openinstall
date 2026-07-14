# AGENTS.md

## Project

OpenInstall — a cross-distro Linux app installer. Thin wrapper around native package managers (apt, pacman, dnf, zypper, pkcon). Linux-only, Rust 1.85+.

## Workspace layout

```
crates/
  installer-core/      → types, adapters, verification, runtime (the real logic)
  installer-cli/       → CLI binary `installer` (20+ commands)
  installer-gui/       → GTK4 + libadwaita GUI (Linux only, optional feature)
  installer-bootstrapper/ → tiny download + launch entry point
```

Dependency chain: cli/bootstrapper → core → (adapters). GUI is a feature flag on cli, default ON.

## Commands

```bash
# Build
cargo build --release
cargo build -p installer-cli --features gui    # with GUI (Linux)
cargo build -p installer-cli --no-default-features  # CLI only, needed for aarch64

# Test (CI tests core, cli, bootstrapper — NOT gui)
cargo test -p installer-core
cargo test -p installer-cli
cargo test -p installer-bootstrapper

# Lint & format (CI runs these, must pass)
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

**Order matters for CI:** clippy and fmt are strict (`-D warnings`, `--check`). Fix clippy warnings before pushing.

## Key quirks

- **GUI is Linux-only.** `installer-gui` depends on `gtk4` and `libadwaita` behind `cfg(target_os = "linux")`. It compiles on non-Linux but is a stub. Do not write GUI code assuming it runs anywhere but Linux.
- **aarch64 release builds are CLI-only.** The release workflow builds aarch64 with `--no-default-features` (no GUI) because cross-compiling GTK4 is not supported.
- **`/etc/os-release` detection.** The core reads this file to identify the distro. Tests may need to mock this or run on real Linux.
- **Package manager adapters are small.** Each adapter (~20 lines) implements install/remove/upgrade for one package manager. Pattern is in `crates/installer-core/src/adapters/`.
- **Blocking HTTP.** Uses `reqwest` with `blocking` feature — not async. All network calls are synchronous.
- **Ed25519 signatures via `ring`.** Signature verification in `crates/installer-core/src/signature.rs`.
- **Manifest format** is JSON with distro-keyed package URLs. See `docs/manifest.md` and `openinstall.json` for examples.
- **Error handling.** CLI uses `Result<(), String>` (`.map_err(|e| e.to_string())`). Core uses `thiserror` enums (`InstallerError`, `ManifestError`, `SignatureError`). New core errors go in the enum, not raw strings.
- **Tests are inline.** Every crate has `#[cfg(test)] mod tests` at the bottom. No separate `tests/` dirs. Filesystem tests use `std::env::temp_dir()` prefixed with `openinstall-test-`.
- **GUI conditional compilation.** GUI code is gated on `#[cfg(all(feature = "gui", target_os = "linux"))]`. It compiles elsewhere but is a stub. Never call GUI functions outside this gate.
- **Commit style.** Conventional: `fix:`, `feat:`, `ci:`, `fix(release):`, `fix(clippy):`.

## Release profile

Aggressive optimization in workspace `Cargo.toml`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`. Builds are slow but small. If you're iterating on compile speed, build with `cargo build` (debug profile) instead.

## Adding a new adapter

Implement `PackageAdapter` trait in `crates/installer-core/src/adapters/`, add module to `mod.rs`, add variant to `PackageManager` enum in `environment.rs`, add arm to `adapter_for()`. ~20 lines. See `docs/architecture.md` for the trait signature.

## Delegation to subagents

**Always use cavecrew subagents for multi-step work.** Do not do investigation + edit + review in one context window — it eats tokens fast.

**Every subagent works in its own git worktree.** Main thread never writes code directly.

| Task | Agent |
|---|---|
| "Where is X defined / what calls Y" | `cavecrew-investigator` |
| Surgical edit, ≤2 files, scope obvious | `cavecrew-builder` |
| Review diff for bugs | `cavecrew-reviewer` |
| New feature / 3+ files | vanilla `task` |

Chaining: **locate → fix → verify.** Investigator finds sites, builder edits, reviewer checks.

### Worktree flow

```bash
# create worktree for agent
git worktree add ../openinstall-<branch> -b <branch>

# agent works in its worktree, commits there

# main thread merges after review
git merge <branch>
git worktree remove ../openinstall-<branch>
```

### Main thread = team lead

Main agent does NOT write code. Its job:

1. **Research** — search the web for APIs, crates, patterns before delegating
2. **Coordinate** — give clear task to subagent (what to do, which files, expected output)
3. **Review** — run `cavecrew-reviewer` on the diff
4. **Merge** — merge worktree branch into main
5. **Push** — push to GitHub
6. **Monitor** — watch CI workflow, fix if broken
7. **Version bump** — update version in `Cargo.toml` workspace, `openinstall.json`, and any other references before release

## When unsure — search the web

If you don't know a Rust API, crate, or framework detail — **websearch first**, then code. Do not guess at APIs. Do not hallucinate function signatures. One search saves a broken build.

## Code style — ponytail mode

Before writing any code, stop at the first rung that holds:

1. **Does this need to exist?** → no: skip it (YAGNI)
2. **Already in this codebase?** → reuse it, don't rewrite
3. **Stdlib does it?** → use it
4. **Native platform feature?** → use it
5. **Installed dependency?** → use it
6. **Есть ли в интернете готовый модуль?** → use it
7. **One line?** → one line
8. **Only then:** the minimum that works

Rules:

- Lazy about the solution, never about reading (read the code first, trace the real flow)
- Never cut: validation, error handling, security, accessibility
- The code ends up small because it is necessary, not golfed
- **Mark shortcuts:** `// ponytail: <ceiling>, upgrade path when <condition>`
- **One guard > many guards.** Fix in the shared function, not every caller.
- **No boilerplate.** No "for later" scaffolding.

Pattern: `[code] → skipped: [X], add when [Y].`
