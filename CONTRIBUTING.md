# Contributing to OpenInstall

Thanks for considering a contribution. Here's everything you need to know.

## Quick start

```bash
git clone https://github.com/bejiihiu/openinstall.git
cd openinstall
cargo build
cargo test -p installer-core
```

You need **Rust 1.85+** and **Linux** (or WSL for basic CLI testing).

## Project structure

```
crates/
  installer-core/          types, adapters, verification, runtime
  installer-cli/           CLI binary (20+ commands)
  installer-gui/           GTK4 + libadwaita GUI (Linux only)
  installer-bootstrapper/  tiny download + launch entry point
```

Dependency chain: `cli/bootstrapper → core → (adapters)`. GUI is a feature flag on CLI.

## Adding a new package manager adapter

This is the easiest way to contribute. Each adapter is ~20 lines.

1. Look at an existing adapter: `crates/installer-core/src/adapters/`
2. Implement the `PackageAdapter` trait for your package manager
3. Add your module to `mod.rs`
4. Add a variant to `PackageManager` enum in `environment.rs`
5. Add an arm to `adapter_for()`
6. Write a test

That's it. If your package manager can install/remove/upgrade packages via CLI, it qualifies.

## Good first issues

Look for issues labeled [`good first issue`](https://github.com/bejiihiu/openinstall/labels/good%20first%20issue) — these are scoped, well-defined tasks perfect for a first contribution.

Some examples:
- Add a new adapter for a package manager
- Improve error messages
- Add tests for edge cases
- Documentation improvements

## Code style

- **Rust edition 2021**, no `unsafe` unless absolutely necessary
- **No new dependencies** unless the value is clear. Check if stdlib or existing deps cover it.
- **Tests are inline** — `#[cfg(test)] mod tests` at the bottom of the file. No separate `tests/` dirs.
- **Error handling** — core uses `thiserror` enums (`InstallerError`, `ManifestError`, `SignatureError`). CLI maps to `String` at the boundary.
- **Blocking HTTP** — uses `reqwest` with `blocking` feature. All network calls are synchronous.
- **Run before submitting:**

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
cargo test -p installer-core
cargo test -p installer-cli
```

## Commit style

Conventional commits: `fix:`, `feat:`, `ci:`, `docs:`, `refactor:`

```
feat: add snap adapter
fix: resolve cache race condition on slow connections
docs: update manifest format examples
```

## Pull request process

1. Fork the repo, create a branch: `feat/snap-adapter` or `fix/cache-race`
2. Make your changes, add tests
3. Run clippy + fmt + tests (see above)
4. Open a PR with a clear description of **what** and **why**
5. Wait for review — usually within a few days

## Reporting bugs

Open an issue using the bug report template. Include:
- Your distro and version
- How to reproduce the issue
- Expected vs actual behavior
- Output of `installer detect`

## Questions?

Open a discussion or an issue — no question is too small.
