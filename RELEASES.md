# Release Strategy

This workspace uses cargo-dist for automated releases with **independent versioning** for CLI and server components.

## Why Independent Versions?

The CLI and server have different release cadences:
- **CLI**: User-facing tool that may get frequent UX improvements, bug fixes, new features
- **Server**: Backend API that changes less frequently, focused on stability

By maintaining separate versions, we can:
- Release CLI updates without rebuilding/re-releasing the server
- Keep release notes focused and relevant
- Allow users to update only what they need

## Release Process

### CLI Release

1. **Bump the version** in `cli/Cargo.toml`:
   ```toml
   [package]
   version = "0.2.1"  # or whatever the new version is
   ```

2. **Update CHANGELOG.md** with CLI changes

3. **Create a git tag** using the package name prefix:
   ```bash
   git tag jot-cli-v0.2.1
   git push origin jot-cli-v0.2.1
   ```

4. **GitHub Actions** will automatically:
   - Build binaries for all platforms (Linux, macOS, Windows)
   - Create installers (`jot-cli-installer.sh` and `.ps1`)
   - Create a GitHub Release titled "jot-cli v0.2.1"
   - Generate release notes from CHANGELOG.md

### Server Release

1. **Bump the version** in `server/Cargo.toml`:
   ```toml
   [package]
   version = "0.3.0"  # or whatever the new version is
   ```

2. **Update CHANGELOG.md** with server changes

3. **Create a git tag** using the package name prefix:
   ```bash
   git tag jot-server-v0.3.0
   git push origin jot-server-v0.3.0
   ```

4. **GitHub Actions** will automatically build and release

### Synchronized Release (Both CLI and Server)

If you need to release both at once (rare):

1. Bump versions in both `cli/Cargo.toml` and `server/Cargo.toml` to the **same version**
2. Update CHANGELOG.md with all changes
3. Create a simple version tag (no package prefix):
   ```bash
   git tag v0.4.0
   git push origin v0.4.0
   ```
4. This will create **one release** with both CLI and server binaries

## Tag Format Reference

cargo-dist recognizes these tag patterns:

| Tag Format | Releases | Example |
|------------|----------|---------|
| `jot-cli-v0.2.1` | Only CLI | `jot-cli-v0.2.1` |
| `jot-cli/0.2.1` | Only CLI | `jot-cli/0.2.1` |
| `jot-server-v0.3.0` | Only server | `jot-server-v0.3.0` |
| `jot-server/0.3.0` | Only server | `jot-server/0.3.0` |
| `v0.4.0` or `0.4.0` | All matching packages | `v1.0.0` |

**Note**: The package-specific tags only work when the packages have **different versions**. If CLI and server both have version `0.2.0`, any tag will release both.

## Installation

Users can install via the generated installers:

```bash
# CLI (installs as 'jot', not 'jot-cli')
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/josefjura/jot/releases/download/jot-cli-v0.2.0/jot-cli-installer.sh | sh

# Server
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/josefjura/jot/releases/download/jot-server-v0.2.0/jot-server-installer.sh | sh
```

Or download pre-built binaries directly from the GitHub Releases page.

## Binary Names

- **CLI**: Installed as `jot` (not `jot-cli`)
- **Server**: Installed as `jot-server`

The CLI package is named `jot-cli` internally for workspace organization, but the actual binary is named `jot` for user convenience.
