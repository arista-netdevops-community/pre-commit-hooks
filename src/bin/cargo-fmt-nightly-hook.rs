use std::process::ExitCode;

use arista_pre_commit_hooks::run_cargo;
use arista_pre_commit_hooks::Toolchain;

fn main() -> ExitCode {
    run_cargo("fmt", Toolchain::Nightly)
}
