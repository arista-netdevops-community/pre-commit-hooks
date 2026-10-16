use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "cargo-about-hook-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("crate/src")).unwrap();
        fs::write(
            path.join("crate/Cargo.toml"),
            "[package]\nname = \"license-fixture\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n",
        )
        .unwrap();
        fs::write(path.join("crate/src/lib.rs"), "").unwrap();
        fs::write(
            path.join("crate/Cargo.lock"),
            "version = 4\n\n[[package]]\nname = \"license-fixture\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(path.join("about.toml"), "accepted = [\"MIT\"]\n").unwrap();
        fs::write(
            path.join("licenses.hbs"),
            "{{#each licenses}}{{id}}:{{#each used_by}}{{crate.name}}{{/each}}\n{{text}}{{/each}}",
        )
        .unwrap();

        // A subprocess-based wrapper would fail if it tried to execute this CLI.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::create_dir(path.join("bin")).unwrap();
            let cli = path.join("bin/cargo-about");
            fs::write(&cli, "#!/bin/sh\nexit 99\n").unwrap();
            fs::set_permissions(cli, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self(path)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-about-hook"));
        command
            .current_dir(&self.0)
            .env("CARGO_NET_OFFLINE", "true");
        #[cfg(unix)]
        {
            let search_path = std::env::var_os("PATH").unwrap_or_default();
            let paths =
                std::iter::once(self.0.join("bin")).chain(std::env::split_paths(&search_path));
            command.env("PATH", std::env::join_paths(paths).unwrap());
        }
        command
    }

    fn generate(&self, extra_args: &[&str]) -> Output {
        self.command()
            .args([
                "generate",
                "--locked",
                "--manifest-path",
                "crate/Cargo.toml",
                "licenses.hbs",
            ])
            .args(extra_args)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generates_licenses_without_external_cli_and_finds_parent_config() {
    let fixture = Fixture::new();
    let output = fixture.generate(&["-o", "THIRD_PARTY_LICENSES.txt"]);
    assert_success(&output);
    assert!(output.stdout.is_empty());
    let text = fs::read_to_string(fixture.0.join("THIRD_PARTY_LICENSES.txt")).unwrap();
    assert!(text.starts_with("MIT:license-fixture\n"), "{text}");
    assert!(text.contains("Permission is hereby granted"), "{text}");

    let second = fixture.generate(&["-o", "THIRD_PARTY_LICENSES.txt"]);
    assert_success(&second);
    assert_eq!(
        fs::read_to_string(fixture.0.join("THIRD_PARTY_LICENSES.txt")).unwrap(),
        text
    );
}

#[test]
fn defaults_to_current_manifest_and_supports_stdout() {
    let fixture = Fixture::new();
    let output = fixture
        .command()
        .current_dir(fixture.0.join("crate"))
        .args(["generate", "--locked", "../licenses.hbs"])
        .output()
        .unwrap();
    assert_success(&output);
    let explicit = fixture.generate(&["--output-file", "-"]);
    assert_success(&explicit);
    assert_eq!(output.stdout, explicit.stdout);
    assert!(String::from_utf8_lossy(&output.stdout).contains("MIT:license-fixture"));
}

#[test]
fn rejected_license_fails_without_overwriting_output() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("about.toml"),
        "accepted = [\"Apache-2.0\"]\n",
    )
    .unwrap();
    let path = fixture.0.join("output.txt");
    fs::write(&path, "previous licenses").unwrap();
    let output = fixture.generate(&["-o", "output.txt"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unable to generate output"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(path).unwrap(), "previous licenses");
}

#[test]
fn invalid_config_fails_with_diagnostics() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("about.toml"),
        "accepted = [\"not-a-license\"]\n",
    )
    .unwrap();
    let output = fixture.generate(&[]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("about.toml"), "{stderr}");
    assert!(stderr.contains("failed to deserialize"), "{stderr}");
}

#[test]
fn locked_generation_does_not_create_a_missing_lockfile() {
    let fixture = Fixture::new();
    let path = fixture.0.join("crate/Cargo.lock");
    fs::remove_file(&path).unwrap();
    let output = fixture.generate(&[]);
    assert!(!output.status.success());
    assert!(!path.exists());
}
