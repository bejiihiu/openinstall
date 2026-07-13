# OpenInstall

A Linux app installer that works across distros. One CLI, one GUI — apt, dnf, pacman, zypper, all the same way.

The problem: you go to download an app on Linux and get six choices, none of which you recognize. Or you copy-paste a curl pipe from a README and hope for the best. OpenInstall is a thin wrapper around your system's native package manager. It figures out what distro you're on, downloads the right package, verifies the signature, then hands it to apt or pacman or whatever you've got.

No new package format. No containers. Just a better first impression.

---

## Install OpenInstall

```bash
curl -sSf https://raw.githubusercontent.com/bejiihiu/openinstall/main/scripts/install.sh | sh
```

Or via the URI itself once installed:

```
openinstall://openinstall?m=https://raw.githubusercontent.com/bejiihiu/openinstall/main/openinstall.json
```

You can also build from source:

```bash
cargo build --release -p installer-cli
cp target/release/installer ~/.local/bin/
```

## Supported distros

| Distro | Package manager | Slot key |
|--------|----------------|----------|
| Arch Linux / Manjaro / EndeavourOS | pacman | `arch` |
| Ubuntu / Debian / Mint / Pop!_OS | apt | `ubuntu` |
| Fedora / RHEL / CentOS | dnf | `fedora` |
| openSUSE | zypper | `opensuse` |
| Anything with PackageKit | pkcon | `fallback` |
| Anything else | — | `fallback` (AppImage, static binary) |

## Requirements

- **Linux** (tested on x86_64, aarch64)
- **A package manager** from the table above (or PackageKit)
- **curl/wget** (for the bootstrapper)
- **GTK4 + libadwaita** (only for `installer gui`)

## Quick start

```bash
# detect your system
installer detect

# install from a local manifest
installer install ./cursor.json

# verify integrity + signature first
installer verify ./cursor.json

# install directly from a URL via URI
installer openinstall://cursor?m=https://example.com/cursor.json

# register the app as a URI handler for your scheme
installer uri register Cursor /usr/bin/cursor openinstall
```

## End-to-end: developer to user

### 1. Developer publishes a manifest

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

### 2. Host the manifest and packages somewhere reachable

A GitHub release, your own CDN, or a static server.

### 3. User installs with one command

```bash
# via manifest URL directly
installer install https://example.com/cursor.json

# via install URI (browser link → installer)
installer openinstall://cursor?m=https://example.com/cursor.json
```

The installer downloads the manifest, picks the right package for the user's distro, verifies sha256 and signature (if present), then hands it to the system package manager.

### 4. (Optional) Register a URI handler

After installation, register the app as a handler so clicking `openinstall://cursor` in a browser opens the installed app:

```bash
installer uri register Cursor /usr/bin/cursor openinstall
```

## CLI

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

# URI subcommands
installer uri <scheme://app>                        parse URI and print details
installer uri <scheme://app?m=manifest_url>         parse and install from manifest URL
installer uri desktop-entry <name> <path> [scheme]  generate a .desktop file
installer uri register <name> <path> [scheme]       register URI handler in the system
installer uri help                                  show URI help

# Direct URI (same as `installer uri ...` but as top-level command)
installer openinstall://cursor?m=https://example.com/manifest.json

installer signature verify <sig> <file>             check ed25519 signature
```

### URI scheme

Three supported schemes:

- `openinstall://app_id`
- `openinstaller://app_id`
- `linuxinstall://app_id`

Query parameters:

| Param | Description |
|-------|-------------|
| `m` | Manifest URL (short form) |
| `manifest` | Manifest URL (full name) |

Example:

```
openinstall://cursor?m=https://example.com/manifest.json
```

If `?m=` or `?manifest=` is present, the installer downloads the manifest and runs the full install flow. Without it, the installer just prints the parsed components.

## GUI

```bash
# if you have gtk4 and libadwaita installed
installer gui ./manifest.json
```

Shows the manifest details in a clean Adwaita window. Verify, install, remove, rollback — all clickable. The language follows your system locale (English and Russian right now; more translations welcome).

## Bootstrapper

For when you want users to just click and go. The bootstrapper grabs a manifest from a URL, picks the right package for the user's distro, downloads it, verifies, and hands off to the GUI.

```bash
# headless — downloads and installs silently
installer-bootstrapper https://example.com/app.json --headless

# with GUI — downloads then opens the installer window
installer-bootstrapper https://example.com/app.json
```

## How it works

Three crates, one dependency chain:

```
installer-core     →  types, adapters, verification, installer runtime
installer-cli      →  CLI (20+ commands) + optional GUI (`--features gui`)
installer-bootstrapper →  tiny entry point for download + launch
```

The core detects the environment by reading `/etc/os-release` and checking which binaries (`pacman`, `apt-get`, `dnf`, `zypper`, `pkcon`) exist on the PATH. It then picks a matching package from the manifest, downloads it, verifies the sha256 and ed25519 signature, and runs the native package manager under the hood.

Every package manager has its own adapter. Adding one is about 20 lines.

## Manifest format

```json
{
    "name": "Cursor",
    "publisher": "Anysphere",
    "version": "1.5.0",
    "description": "AI Code Editor",
    "homepage": "https://cursor.sh",
    "license": "MIT",
    "changelog": "See https://cursor.sh/changelog",
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

## Security

- All downloads go through TLS (`reqwest` + `rustls`)
- Network requests have timeouts: 15s for manifests, 30s connect + 120s total for packages
- SHA256 is checked before the package touches the package manager
- Ed25519 signatures are verified with `ring`
- If a signature is present and invalid, the install button disables and a red banner shows up
- If no signature is provided, the UI says so — no silent trust

## Building

```bash
cargo build --release
# or just specific crates
cargo build -p installer-core
cargo build -p installer-cli --features gui   # include GUI (Linux only)
```

Tests:

```bash
cargo test -p installer-core
cargo test -p installer-cli
```

The GUI requires GTK4 and libadwaita development headers on your system (`libgtk-4-dev` and `libadwaita-1-dev` on Debian/Ubuntu, `gtk4` and `libadwaita` on Arch, etc).

## What this isn't

It's not a new package manager. It's not a container runtime. It's not an app store. It's a frontend that reads a JSON file, downloads stuff, checks signatures, and calls your system's package manager. Nothing more.

## What's next

There's a roadmap in [docs/](docs/). Short version: live progress bars during install, a GitHub Action for auto-publishing manifests, and better post-install UX (launch, open folder).

## License

[MIT](./LICENSE)

## Support

I'm not asking for donations or a subscription for every sneeze this project makes.

If you run a Telegram channel, a Discord server, or just know people who'd find this useful — tell them about OpenInstall. That's already a big deal for me.

Honestly, I didn't start this for attention. I just wanted my friend — who switched to Linux — to be able to install Discord with one click instead of googling what `.deb` is, what Flatpak is, what AUR is, and why there are five different ways to install one app

If the project helped you — drop a ⭐ on GitHub. Helps other people find it.

And if you want to support me personally — buy me a coffee or a meal for one evening. I'm a student too, and sometimes that kind of help goes a long way <3

Thanks to everyone making Linux a little friendlier.

---

Also read: [HONESTY.md](HONESTY.md) — why this project exists and why I'm pissed off about app installation on Linux 😄
