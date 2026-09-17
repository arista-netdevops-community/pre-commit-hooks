# pre-commit-hooks

Rust-based hooks for pre-commit and pre-commit.ci.

## Hooks

### cargo-fmt-nightly

Runs `cargo fmt` with `RUSTUP_TOOLCHAIN=nightly`.

Example:

```yaml
- repo: https://github.com/arista-netdevops-community/pre-commit-hooks
  rev: <rev>
  hooks:
    - id: cargo-fmt-nightly
      args:
        - --all
        - --
        - --config
        - unstable_features=true,group_imports=StdExternalCrate,imports_granularity=Item
```

### cargo-fmt

Runs `cargo fmt` with the target repository's selected Rust toolchain.

Example:

```yaml
- repo: https://github.com/arista-netdevops-community/pre-commit-hooks
  rev: <rev>
  hooks:
    - id: cargo-fmt
      args: ["--all"]
```

### cargo-check

Runs `cargo check` with the target repository's selected Rust toolchain.

Example:

```yaml
- repo: https://github.com/arista-netdevops-community/pre-commit-hooks
  rev: <rev>
  hooks:
    - id: cargo-check
      args: ["--locked"]
```

### cargo-clippy

Runs `cargo clippy` with the target repository's selected Rust toolchain.

Example:

```yaml
- repo: https://github.com/arista-netdevops-community/pre-commit-hooks
  rev: <rev>
  hooks:
    - id: cargo-clippy
      args: ["--locked", "--all-targets", "--all-features"]
```

### cargo-deny

Runs `cargo-deny` directly. The hook installs and invokes the version pinned as a
Cargo CLI dependency in pre-commit's isolated environment.

Example:

```yaml
- repo: https://github.com/arista-netdevops-community/pre-commit-hooks
  rev: <rev>
  hooks:
    - id: cargo-deny
      args: ["--locked", "--all-features", "check"]
```

### cargo-about

Runs `cargo-about` directly. The hook installs and invokes the version pinned as
a Cargo CLI dependency in pre-commit's isolated environment.

Example:

```yaml
- repo: https://github.com/arista-netdevops-community/pre-commit-hooks
  rev: <rev>
  hooks:
    - id: cargo-about
      name: Generate third-party Rust licenses
      args:
        - generate
        - --locked
        - -o
        - pyavd_utils/THIRD_PARTY_LICENSES.txt
        - about-text.hbs
```

Do not pass `--fail` when using `cargo-about` to regenerate tracked files in
pre-commit. Let pre-commit report changed files; `--fail` is for making
license-resolution issues fail the `cargo about generate` process.
