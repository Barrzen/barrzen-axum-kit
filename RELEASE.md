# Barrzen Axum Kit Release Strategy

This document outlines the versioning and release process for the `barrzen-axum-kit` workspace.

## Versioning

We follow [SemVer](https://semver.org) for all crates in the workspace.
All crates in the workspace (`barrzen-axum-core`, `barrzen-axum-infra`, etc.) should share the SAME version number to ensure compatibility.

## Release Process

1. **Update Versions**:
   Update the `version` field in `Cargo.toml` of all crates:
   - `barrzen-axum-kit/Cargo.toml` (workspace.package)
   - Crates inheriting version will update automatically.

2. **Update Changelog**:
   Update `CHANGELOG.md` (if present) with new changes.

3. **Commit & Tag**:

   ```bash
   git commit -am "chore: release v0.2.0"
   git tag v0.2.0
   git push origin main --tags
   ```

4. **Publish**:
   Publish crates in dependency order:
   1. `barrzen-axum-core`
   2. `barrzen-axum-infra`
   3. `barrzen-axum-obs`
   4. `barrzen-axum-openapi`

   ```bash
   cargo publish -p barrzen-axum-core
   # ...
   ```

## Downstream Usage

Consumers should depend on specific versions of the kit components:

```toml
[dependencies]
barrzen-axum-core = "0.2"
barrzen-axum-infra = { version = "0.2", features = ["db", "cache-redis"] }
```

## Branching

- `main`: Development branch (unstable).
- `vX.Y`: Release branches (optional, for maintenance).
