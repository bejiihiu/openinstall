# OpenInstall

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/bejiihiu/openinstall/ci.yml?branch=main&label=CI">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img src="https://img.shields.io/github/license/bejiihiu/openinstall">
  <img src="https://img.shields.io/github/stars/bejiihiu/openinstall">
  <img src="https://img.shields.io/github/v/release/bejiihiu/openinstall">
  <img src="https://img.shields.io/badge/Linux-only-blue">
  <a href="https://github.com/bejiihiu/openinstall/blob/main/CONTRIBUTING.md"><img src="https://img.shields.io/badge/contributions-welcome-brightgreen"></a>
</p>

A Linux app installer that works across distros. One CLI, one GUI — apt, dnf, pacman, zypper, all the same way.

The problem: you go to download an app on Linux and get six choices, none of which you recognize. Or you copy-paste a curl pipe from a README and hope for the best. OpenInstall is a thin wrapper around your system's native package manager. It figures out what distro you're on, downloads the right package, verifies the signature, then hands it to apt or pacman or whatever you've got.

No new package format. No containers. Just a better first impression.

---

## Install

```bash
curl -sSf https://raw.githubusercontent.com/bejiihiu/openinstall/main/scripts/install.sh | sh
```

That's it. The script detects your architecture (x86_64 / aarch64), downloads the right binary, and registers the desktop entry + URI handlers automatically. After install, `openinstall://` links in your browser will open directly in OpenInstall.

<details>
<summary>Other install methods</summary>

### URI (once OpenInstall is installed)

```
openinstall://openinstall?m=https://raw.githubusercontent.com/bejiihiu/openinstall/main/openinstall.json
```

### Build from source

```bash
cargo build --release -p installer-cli --features gui
cp target/release/installer ~/.local/bin/
installer gui --register-desktop   # register in app menu + URI handlers
```

### Self-update

```bash
installer self-update
```

</details>

## Quick start

```bash
installer detect                                    # show your distro, arch, package manager
installer install https://example.com/app.json      # install an app from a manifest
installer gui                                        # open the graphical installer
```

## Supported distros

| Distro | Package manager | Slot key |
|--------|----------------|----------|
| Arch Linux / Manjaro / EndeavourOS | pacman | `arch` |
| Ubuntu / Debian / Mint / Pop!_OS | apt | `ubuntu` |
| Fedora / RHEL / CentOS | dnf | `fedora` |
| openSUSE | zypper | `opensuse` |
| Anything with PackageKit | pkcon | `fallback` |
| Anything else | — | `fallback` (AppImage) |

## For app developers

### 1. Publish a manifest

```bash
installer publish \
  --name "Cursor" \
  --publisher "Anysphere" \
  --version "1.5.0" \
  --description "AI Code Editor" \
  --ubuntu ./cursor-amd64.deb \
  --arch ./cursor-x86_64.pkg.tar.zst \
  --output ./cursor.json
```

### 2. Host it (GitHub release, CDN, your server)

### 3. Users install with one command

```bash
installer install https://example.com/cursor.json
```

Or via a browser link:

```
openinstall://cursor?m=https://example.com/cursor.json
```

<details>
<summary>Full CLI reference</summary>

```
installer detect                                    print distro/arch/package manager
installer validate <manifest>                       check manifest is correct
installer select <manifest>                         show which package matches your system
installer show <manifest>                           print everything about an app
installer verify <manifest>                         download + check sha256 + signature
installer install <manifest>                        download, verify, install
installer remove <manifest>                         uninstall
installer update <manifest>                         same as install
installer reinstall <manifest>                      force reinstall
installer rollback <manifest>                       go back to previous cache version
installer history <manifest>                        installation history
installer cache clear                               nuke the cache
installer cache info                                show cache size
installer publish --name ... (see above)            generate a manifest
installer serve <manifest> [addr]                   serve /app/latest on HTTP
installer gui [manifest]                            launch graphical installer (Linux only)
installer gui --register-desktop                    add to application menu + URI handlers
installer self-update                                download and replace itself

# URI subcommands
installer uri <scheme://app>                        parse URI and print details
installer uri <scheme://app?m=manifest_url>         parse and install from manifest URL
installer uri desktop-entry <name> <path> [scheme]  generate a .desktop file
installer uri register <name> <path> [scheme]       register URI handler in the system

# Direct URI (same as `installer uri ...` but as top-level command)
installer openinstall://cursor?m=https://example.com/manifest.json

installer signature verify <sig> <file>             check ed25519 signature
```

</details>

## Manifest format

```json
{
    "name": "Cursor",
    "publisher": "Anysphere",
    "version": "1.5.0",
    "description": "AI Code Editor",
    "homepage": "https://cursor.sh",
    "license": "MIT",
    "packages": {
        "arch": "https://example.com/cursor.pkg.tar.zst",
        "ubuntu": "https://example.com/cursor.deb",
        "fedora": "https://example.com/cursor.rpm",
        "opensuse": "https://example.com/cursor.rpm",
        "fallback": "https://example.com/cursor.AppImage"
    },
    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "signature": "ed25519:<public_key_hex>:<signature_hex>"
}
```

See [docs/manifest.md](docs/manifest.md) for details.

## How it works

Three crates, one dependency chain:

```
installer-core     →  types, adapters, verification, installer runtime
installer-cli      →  CLI (20+ commands) + optional GUI (`--features gui`)
installer-bootstrapper →  tiny entry point for download + launch
```

The core detects the environment by reading `/etc/os-release` and checking which binaries (`pacman`, `apt-get`, `dnf`, `zypper`, `pkcon`) exist on the PATH. It then picks a matching package from the manifest, downloads it, verifies the sha256 and ed25519 signature, and runs the native package manager under the hood.

Every package manager has its own adapter. Adding one is about 20 lines.

## Security

- All downloads go through TLS (`reqwest` + `rustls`)
- Network requests have timeouts: 15s for manifests, 30s connect + 120s total for packages
- SHA256 is checked before the package touches the package manager
- Ed25519 signatures are verified with `ring`

## Requirements

- **Linux** (x86_64 or aarch64)
- **A package manager** from the table above (or PackageKit)
- **curl** (for the install script)
- **GTK4 + libadwaita** (only for `installer gui` — optional)

## Building

```bash
cargo build --release
# or specific crates
cargo build -p installer-cli --features gui   # with GUI (Linux only)
```

Tests:

```bash
cargo test -p installer-core
cargo test -p installer-cli
```

The GUI requires GTK4 and libadwaita development headers (`libgtk-4-dev` and `libadwaita-1-dev` on Debian/Ubuntu, `gtk4` and `libadwaita` on Arch).

## What this isn't

It's not a new package manager. It's not a container runtime. It's not an app store. It's a frontend that reads a JSON file, downloads stuff, checks signatures, and calls your system's package manager. Nothing more.

## Contributing

We'd love your help. Here's how to get started:

1. **Easy entry** — look for [`good first issue`](https://github.com/bejiihiu/openinstall/labels/good%20first%20issue) labels. Adding a new package manager adapter is ~20 lines.
2. **Read the guide** — see [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code style, and PR process.
3. **Report bugs** — use the [bug report template](https://github.com/bejiihiu/openinstall/issues/new?template=bug_report.yml).
4. **Request features** — use the [feature request template](https://github.com/bejiihiu/openinstall/issues/new?template=feature_request.yml).

Even small PRs help — fix a typo, improve an error message, add a test. Every contribution counts.

## Roadmap

- [x] Cross-distro CLI installer (apt, pacman, dnf, zypper, pkcon)
- [x] Manifest format with SHA256 + ed25519 signatures
- [x] GTK4 + libadwaita GUI
- [x] URI handler (`openinstall://` links)
- [x] Self-update mechanism
- [ ] Snap adapter
- [ ] Nix adapter
- [ ] Flatpak adapter (as install source)
- [ ] Batch install (multiple manifests)
- [ ] Offline install support
- [ ] Shell completion (bash, zsh, fish)
- [ ] Man page

See [open issues](https://github.com/bejiihiu/openinstall/issues) for what's being worked on. Pick something from the roadmap and claim it — open an issue so others know it's taken.

## License

[MIT](./LICENSE)

## Support

If you run a Telegram channel, a Discord server, or just know people who'd find this useful — tell them about OpenInstall.

If the project helped you — drop a ⭐ on GitHub. Helps other people find it.

---

Also read: [HONESTY.md](HONESTY.md) — why this project exists.
