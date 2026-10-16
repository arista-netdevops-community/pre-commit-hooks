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

Generates licenses with the `cargo-about` library linked into `cargo-about-hook`.
The dependency version is maintained in `Cargo.toml`; no separate `cargo-about`
executable or Cargo CLI installation features are needed. Installing the Rust
hooks requires Rust 1.88 or newer.

The adapter supports only `generate`, with `--locked`, `--manifest-path`,
`-o`/`--output-file`, and a single Handlebars template file. It discovers
`about.toml` beside the manifest or in a parent directory, using cargo-about's
configuration, workarounds, and license-resolution behavior. Output defaults to
stdout; `-o -` also writes to stdout.

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

Let pre-commit report changed files. The adapter uses cargo-about's default
license-resolution behavior and does not expose `--fail` or its other CLI
commands and options.
