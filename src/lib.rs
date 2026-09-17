use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::process::Command;
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Toolchain {
    Default,
    Nightly,
}

pub fn cargo_args<I, S>(subcommand: &str, args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut cargo_args = Vec::from([OsString::from(subcommand)]);
    cargo_args.extend(args.into_iter().map(Into::into));
    cargo_args
}

pub fn run_cargo(subcommand: &str, toolchain: Toolchain) -> ExitCode {
    let args = cargo_args(subcommand, env::args_os().skip(1));
    let mut command = command("cargo", &args);

    if toolchain == Toolchain::Nightly {
        command.env("RUSTUP_TOOLCHAIN", "nightly");
    }

    run(command, &args)
}

/// Runs a CLI installed in the hook environment with the arguments passed to the hook.
///
/// External Cargo CLIs must be invoked directly instead of through `cargo <subcommand>`.
/// Cargo's subcommand lookup may otherwise select an executable outside pre-commit's
/// isolated environment and bypass the version pinned in `additional_dependencies`.
pub fn run_cli(program: &str) -> ExitCode {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    run(command(program, &args), &args)
}

fn command(program: &str, args: &[OsString]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

fn run(mut command: Command, args: &[OsString]) -> ExitCode {
    let program = command.get_program().to_string_lossy().into_owned();

    match command.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!(
                "failed to run `{}`: {error}",
                format_command(&program, args.iter())
            );
            ExitCode::FAILURE
        }
    }
}

fn format_command<'a>(program: &str, args: impl Iterator<Item = &'a OsString>) -> String {
    let mut parts = Vec::from([program.to_string()]);
    parts.extend(args.map(|arg| format_arg(arg)));
    parts.join(" ")
}

fn format_arg(arg: &OsStr) -> String {
    let value = arg.to_string_lossy();

    if value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || "-_./=,:".contains(char))
    {
        value.into_owned()
    } else {
        format!("{value:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_args_prepends_subcommand() {
        let args = cargo_args("fmt", ["--all", "--", "--check"]);

        assert_eq!(
            args,
            vec![
                OsString::from("fmt"),
                OsString::from("--all"),
                OsString::from("--"),
                OsString::from("--check"),
            ]
        );
    }

    #[test]
    fn cargo_args_preserves_empty_args() {
        assert_eq!(
            cargo_args("check", std::iter::empty::<OsString>()),
            [OsString::from("check")]
        );
    }

    #[test]
    fn command_invokes_external_cli_directly() {
        let args = [OsString::from("--version")];
        let command = command("cargo-deny", &args);

        assert_eq!(command.get_program(), "cargo-deny");
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["--version"]);
    }
}
