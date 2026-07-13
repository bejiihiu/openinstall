# Manifest format

A manifest is a JSON file that tells OpenInstall everything it needs to install an app. You can write one by hand, generate it with `installer publish`, or serve it from an API.

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | App name — used for the package id and desktop entry |
| `publisher` | yes | Developer or company name |
| `version` | yes | Version string. Compared when checking for updates |
| `description` | yes | Short description of the app |
| `packages` | yes | Map of distro → package URL (see below) |
| `homepage` | no | Project website |
| `license` | no | SPDX identifier or URL |
| `changelog` | no | URL or inline text |
| `image` | no | Path or URL to an icon (shown in the GUI) |
| `sha256` | no | Hex-encoded SHA256 of the package file |
| `signature` | no | Ed25519 signature in `ed25519:<pubkey>:<sig>` format |

## Package matrix

The `packages` field maps distribution families to download URLs:

```json
"packages": {
  "arch": "https://.../app.pkg.tar.zst",
  "ubuntu": "https://.../app.deb",
  "fedora": "https://.../app.rpm",
  "opensuse": "https://.../app.rpm",
  "fallback": "https://.../app.AppImage"
}
```

The installer picks the first match:

1. Arch Linux → `arch`
2. Ubuntu / Debian → `ubuntu`
3. Fedora / RHEL / CentOS → `fedora`
4. openSUSE → `opensuse`
5. Everything else → `fallback`

Each value can be a string URL or an object with a URL key:

```json
"ubuntu": { "url": "https://.../app.deb" }
```

## GitHub Releases

If the URL points to a GitHub repository (e.g. `https://github.com/owner/repo`), the installer fetches the latest release and picks the first asset ending with the right extension (`.deb` for Ubuntu, `.rpm` for Fedora/openSUSE, `.pkg.tar.zst` for Arch, `.AppImage` for fallback).

## API server

You can serve manifests over HTTP with `installer serve`:

```bash
installer serve ./app-manifest.json 127.0.0.1:8080
```

The endpoint is `GET /app/latest`:

```json
GET /app/latest
200 OK
Content-Type: application/json

{
  "version": "1.5.0",
  "packages": { "ubuntu": "https://..." }
}
```

## History

The installer keeps a history of every installation in `<cache>/history.json`. Each entry records the package id, version, package manager, staged path, sha256, and timestamp. You can view it with `installer history <manifest>` or through the GUI.
