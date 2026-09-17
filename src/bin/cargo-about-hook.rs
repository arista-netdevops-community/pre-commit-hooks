use std::process::ExitCode;

use arista_pre_commit_hooks::run_cli;

fn main() -> ExitCode {
    run_cli("cargo-about")
}
