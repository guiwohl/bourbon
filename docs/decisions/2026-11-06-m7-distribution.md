# M7 Distribution

Decided by Claude during the autonomous run; review at the UI pass.

## Decided

| Topic | Decision |
|---|---|
| Desktop entry | `packaging/bourbon.desktop`: Exec `bourbon-app`, Icon `bourbon`, StartupWMClass `bourbon`, validated with `desktop-file-validate` (only a "more than one main category" hint, kept as asked) |
| Single instance | The GUI connects to a Unix socket `$XDG_RUNTIME_DIR/<app id>.sock` (fallback `/tmp/<app id>-<uid>.sock`). A live instance receives `focus\n` and the new process exits 0. A refused connection means a stale socket: remove, bind, listen on a std thread |
| Isolation | The app id comes from `Identity`, so `bourbon-dev` and `bourbon` never collide |
| Activation | The listener thread sends on an mpsc channel; a GPUI task polls it every 100 ms and calls `activate_window()`. Unknown messages are ignored |
| Tests | Socket logic is GPUI-free and tested with temp dirs. The GPUI activation step has no headless test; the forwarding (message to channel) is tested |
| Local install | `just install` / `just uninstall` into `~/.local` |
| Packaging files | `packaging/aur/PKGBUILD` and `.SRCINFO` for package `bourbon`; checksum is the `_tarball_sha256` placeholder, filled at release |

## PROPOSAL — the user wants to discuss it before the first release

- SemVer from the workspace `version`; tag `vX.Y.Z` must equal it, the workflow fails otherwise.
- `.github/workflows/release.yml` runs on `v*` tags only: release build, tarball `bourbon-<version>-x86_64-linux.tar.gz`, draft GitHub release.
- The AUR package builds from the GitHub tag tarball.
- Nothing is published until the user pushes a tag. Flow: [../development/release.md](../development/release.md).
