use std::process::ExitCode;

use arista_pre_commit_hooks::{run_cargo, Toolchain};

fn main() -> ExitCode {
    run_cargo("deny", Toolchain::Default)
}
