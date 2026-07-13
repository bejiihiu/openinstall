# Architecture

Three crates, one job. The core library does all the actual work. The CLI and GUI are just frontends.

## Crates

### installer-core

Zero GUI, zero CLI — just package management primitives.

- **Manifest** — parse, validate, resolve packages to environments
- **Environment detection** — reads `/etc/os-release`, checks PATH for package managers
- **Package adapters** — each package manager (apt, dnf, pacman, zypper, packagekit) implements the same trait
- **Installer runtime** — download, verify sha256, verify ed25519, execute package manager commands, track history
- **GitHub integration** — resolves repository URLs to release assets
- **Signature verification** — Ed25519 with the `ring` crate
- **API server** — minimal HTTP server serving `/app/latest`
- **Publish** — manifest generation from CLI flags
- **URI parsing** — `openinstall://` and `linuxinstall://` scheme handlers

```
installer-core/src/
  lib.rs          → Manifest, PackageMatrix, Environment, PackageManager
  adapters.rs     → PackageAdapter trait + Apt/Dnf/Pacman/Zypper/PackageKit
  runtime.rs      → Installer: install, remove, update, rollback, verify, history, cache
  signature.rs    → Ed25519 signature parsing + verification
  github.rs       → GitHub Releases API client
  api.rs          → HTTP server for serving manifests
  publish.rs      → Manifest generation
  desktop.rs      → .desktop file generation
  uri.rs          → Install URI scheme parser
```

### installer-cli

Command-line interface with 20+ commands. One match on the first argument, a bunch of thin wrappers around installer-core functions. Optionally includes the GTK GUI behind `--features gui` (Linux only).

### installer-gui / `installer gui`

GTK4 + LibAdwaita window (merged into installer-cli, accessible via `installer gui`). Four pages:

- **Manifest page** — shows app name, publisher, version, license, package matrix, changelog, signature status. Buttons for install/remove/verify/rollback/reload/cache-info/history.
- **Installing page** — spinner + progress bar + log output.
- **Done page** — launch app, open folder, close.
- **Error page** — error message, close.

Everything async. Clicking install spawns a thread that downloads and runs the package manager; progress updates come back through an mpsc channel and get processed by a glib idle handler.

### installer-bootstrapper

The smallest crate. Takes a manifest URL, downloads it, picks a package, downloads it, and either opens the GUI or runs headless.

## Data flow

```
User clicks Install
  ↓
GUI sends ProgressUpdate::Installing to main thread
  ↓
Thread: install_with_progress(tx)
  → stage_package() → download() → progress updates via tx
  → verify_sha256()
  → verify_signature()
  → run_command_streaming() → log lines via tx
  → append_history()
  → send InstallResult via tx
  ↓
GUI idle handler receives result
  → transitions to Done page
```

## Adding a package manager

Implement the `PackageAdapter` trait:

```rust
struct MyAdapter;

impl PackageAdapter for MyAdapter {
    fn manager(&self) -> PackageManager { PackageManager::Unknown }
    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) { ... }
    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) { ... }
    // ...
}
```

Then add it to `adapter_for()` in adapters.rs. That's it.
