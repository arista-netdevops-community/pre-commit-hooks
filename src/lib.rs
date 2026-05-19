use std::env;
use std::ffi::{OsStr, OsString};
use std::process::{Command, ExitCode};

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
    let mut command = Command::new("cargo");
    command.args(&args);

    if toolchain == Toolchain::Nightly {
        command.env("RUSTUP_TOOLCHAIN", "nightly");
    }

    match command.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!(
                "failed to run `{}`: {error}",
                format_command("cargo", args.iter())
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
}
