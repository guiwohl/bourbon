# Release (proposal)

Proposal, not yet agreed: see [the M7 decision](../decisions/2026-11-06-m7-distribution.md). Nothing here runs until a tag is pushed.

1. Bump `version` in `[workspace.package]` of `Cargo.toml`, merge to `main`.
2. Tag `vX.Y.Z` (must equal the workspace version) and push the tag.
3. `release.yml` checks the tag, builds `--release --locked`, packs `bourbon-<version>-x86_64-linux.tar.gz` (both binaries, `.desktop`, icons, LICENSE, README) and creates a draft GitHub release. Review and publish it by hand.
4. AUR: set `pkgver` in `packaging/aur/PKGBUILD`, replace `_tarball_sha256` with `sha256sum` of the tag tarball (`updpkgsums` or by hand), regenerate with `makepkg --printsrcinfo > .SRCINFO`, lint with `namcap PKGBUILD`, push to the AUR repo.

Local install without a release: `just install`.
