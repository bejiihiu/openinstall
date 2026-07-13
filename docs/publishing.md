# Publishing an app

You've built a Linux app and you want people to install it without reading a wiki page. You need a manifest.

## Using the CLI

The easiest way is `installer publish`:

```bash
installer publish \
  --name "MyApp" \
  --publisher "You" \
  --version "1.0.0" \
  --description "Does a thing" \
  --ubuntu ./myapp-1.0.0-amd64.deb \
  --arch ./myapp-1.0.0-x86_64.pkg.tar.zst \
  --fedora ./myapp-1.0.0-x86_64.rpm \
  --opensuse ./myapp-1.0.0-x86_64.rpm \
  --output ./myapp.json
```

This validates the fields and writes a manifest file. You can also pipe to stdout with `--output -`.

## Hosting

Put the manifest somewhere your users can reach it. A GitHub release, your own CDN, or just a static server. Then users install with:

```bash
installer-bootstrapper https://yourapp.com/myapp.json
```

Or open the GUI:

```bash
installer-bootstrapper https://yourapp.com/myapp.json
```

## GitHub Releases

If you host your packages on GitHub Releases, you don't need to update the manifest URL every release. The installer can resolve the latest release automatically by looking at the GitHub API — just use the repository URL as the package reference:

```json
{
  "packages": {
    "ubuntu": "https://github.com/you/myapp"
  }
}
```

The installer fetches `https://api.github.com/repos/you/myapp/releases/latest` and picks the first asset ending with `.deb`.

## GitHub Action

There's a workflow template at `.github/workflows/publish-manifest.yml`. Configure it in your repo:

```yaml
on:
  release:
    types: [published]
```

On every release, it downloads the release assets, runs `installer publish`, and produces a manifest. You can upload it as a build artifact or attach it back to the release.

## Signing

For the `signature` field, generate an Ed25519 keypair:

```bash
# install openssl or use any ed25519 tool
# then format as: ed25519:<public_key_hex>:<signature_hex>
```

Sign the package file with your private key and put the signature in the manifest. The installer verifies it before running the package manager. If the signature is bad, the GUI shows a red warning and refuses to install.

## Tips

- Include all four distro slots if you can. If not, `fallback` with an AppImage covers everyone else.
- The `sha256` field is checked before installation. It's not optional in practice — include it.
- Keep old versions in the cache. Rollback needs them.
- You don't need a custom API. A static file works fine. The API server (`installer serve`) is for testing.
