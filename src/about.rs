// License-generation orchestration adapted from cargo-about 0.9.2:
// https://github.com/EmbarkStudios/cargo-about/blob/0.9.2/src/cargo-about/generate.rs
// Copyright (c) 2019 Embark Studios. Licensed under the MIT license;
// see LICENSES/cargo-about-MIT.txt.

use std::io::IsTerminal;
use std::io::Write;
use std::sync::Arc;

use anyhow::Context as _;
use cargo_about::licenses;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::term;
use krates::Utf8Path;
use krates::Utf8PathBuf;
use toml_span::Deserialize as _;

#[derive(clap::Args)]
pub struct Args {
    /// Assert that Cargo.lock remains unchanged.
    #[arg(long)]
    locked: bool,
    /// Root Cargo.toml (defaults to Cargo.toml in the current directory).
    #[arg(long)]
    manifest_path: Option<Utf8PathBuf>,
    /// Write the generated licenses to this file (defaults to stdout).
    #[arg(short = 'o', long)]
    output_file: Option<Utf8PathBuf>,
    /// Handlebars template file.
    template: Utf8PathBuf,
}

type Stream = term::termcolor::StandardStream;
type Files = codespan::Files<String>;

pub fn generate(args: Args) -> anyhow::Result<()> {
    let manifest = match args.manifest_path {
        Some(path) => path,
        None => Utf8PathBuf::from_path_buf(std::env::current_dir()?)
            .map_err(|path| anyhow::anyhow!("working directory is not UTF-8: {}", path.display()))?
            .join("Cargo.toml"),
    };
    anyhow::ensure!(
        manifest.is_file(),
        "cargo manifest '{manifest}' does not exist"
    );
    anyhow::ensure!(
        args.template.is_file(),
        "template '{}' is not a file",
        args.template
    );

    let stdout = args.output_file.as_deref().is_none_or(|path| path == "-");
    if stdout {
        anyhow::ensure!(
            !cargo_about::is_powershell_parent(),
            "use --output-file to avoid PowerShell output encoding issues"
        );
    }

    let stream = Stream::stderr(if std::io::stderr().is_terminal() {
        term::termcolor::ColorChoice::Auto
    } else {
        term::termcolor::ColorChoice::Never
    });
    let mut files = Files::new();
    let (config_id, config) = load_config(&manifest, &stream, &mut files)?;
    let mut templates = handlebars::Handlebars::new();
    templates.register_template_file("licenses", &args.template)?;

    let krates = cargo_about::get_all_crates(
        &manifest,
        false,
        false,
        Vec::new(),
        false,
        krates::LockOptions {
            frozen: false,
            locked: args.locked,
            offline: false,
        },
        &config,
        &[],
    )?;
    let store = licenses::store_from_cache()?;
    let client = ureq::Agent::config_builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .unversioned_rustls_crypto_provider(Arc::new(
                    rustls::crypto::ring::default_provider(),
                ))
                .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent();
    let summary = licenses::Gatherer::with_store(Arc::new(store))
        .with_confidence_threshold(0.8)
        .with_max_depth(config.max_depth.map(|depth| depth as usize))
        .gather(&krates, &config, Some(client));

    let mut crate_configs = std::collections::BTreeMap::new();
    for (name, config) in config.crates {
        if krates.krates_by_name(name.clone()).count() != 0 {
            crate_configs.insert(name, config.value);
        } else {
            emit(
                &stream,
                &files,
                &Diagnostic::warning()
                    .with_message("config found for a crate not present in the graph")
                    .with_labels(vec![Label::primary(config_id, config.span)]),
            )?;
        }
    }
    // Match cargo-about generate's default behavior without --fail.
    let resolved = licenses::resolution::resolve(
        &summary,
        &config.accepted,
        &crate_configs,
        &mut files,
        false,
    );
    let input = cargo_about::generate::generate(&summary, &resolved, |diagnostics| {
        for diagnostic in diagnostics {
            let _ = emit(&stream, &files, diagnostic);
        }
    })?;
    let output = templates.render("licenses", &input)?;

    if stdout {
        writeln!(std::io::stdout().lock(), "{output}")?;
    } else if let Some(path) = args.output_file {
        std::fs::write(&path, output)
            .with_context(|| format!("output file '{path}' could not be written"))?;
    }
    Ok(())
}

fn load_config(
    manifest: &Utf8Path,
    stream: &Stream,
    files: &mut Files,
) -> anyhow::Result<(codespan::FileId, licenses::config::Config)> {
    for parent in manifest.ancestors().skip(1) {
        let path = parent.join("about.toml");
        if !path.is_file() {
            continue;
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file '{path}'"))?;
        let id = files.add(&path, contents.clone());
        let mut value = toml_span::parse(&contents).map_err(|error| {
            let _ = emit(
                stream,
                files,
                &Diagnostic::error()
                    .with_message(error.to_string())
                    .with_labels(vec![Label::primary(id, error.span)]),
            );
            anyhow::anyhow!("failed to parse config file '{path}'")
        })?;
        let config = licenses::config::Config::deserialize(&mut value).map_err(|errors| {
            for error in errors.errors {
                let _ = emit(
                    stream,
                    files,
                    &Diagnostic::error()
                        .with_message(error.to_string())
                        .with_labels(vec![Label::primary(id, error.span)]),
                );
            }
            anyhow::anyhow!("failed to deserialize config file '{path}'")
        })?;
        return Ok((id, config));
    }

    log::warn!("no 'about.toml' found, falling back to default configuration");
    Ok((files.add("none", String::new()), Default::default()))
}

fn emit(
    stream: &Stream,
    files: &Files,
    diagnostic: &Diagnostic<codespan::FileId>,
) -> anyhow::Result<()> {
    term::emit_to_io_write(
        &mut stream.lock(),
        &term::Config::default(),
        files,
        diagnostic,
    )?;
    Ok(())
}
