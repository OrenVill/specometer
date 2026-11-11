use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use specometer_core::pipeline::{run_slice, validate_inputs};
use specometer_core::resolver::{HeuristicMode, ResolverConfig};
use specometer_core::SpecometerError;

#[derive(Parser)]
#[command(name = "specometer", version = specometer_core::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Slice(SliceArgs),
    Bind(SliceArgs),
    Validate(ValidateArgs),
}

#[derive(clap::Args)]
struct SliceArgs {
    #[arg(long)]
    spec: PathBuf,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, default_value = "when-no-explicit")]
    heuristic: String,
    #[arg(long)]
    strict: bool,
}

#[derive(clap::Args)]
struct ValidateArgs {
    #[arg(long)]
    spec: PathBuf,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long, default_value = "when-no-explicit")]
    heuristic: String,
    #[arg(long)]
    strict: bool,
}

fn main() {
    if let Err(code) = run() {
        process::exit(code);
    }
}

fn run() -> Result<(), i32> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Slice(args) => {
            execute_slice(&args, false)?;
        }
        Commands::Bind(args) => {
            execute_slice(&args, true)?;
        }
        Commands::Validate(args) => {
            execute_validate(&args)?;
        }
    }
    Ok(())
}

fn execute_slice(args: &SliceArgs, bind: bool) -> Result<(), i32> {
    let cfg = match config_from_args(&args.heuristic, args.strict) {
        Ok(c) => c,
        Err(e) => return exit_from_error(e),
    };
    let spec_src = match fs::read_to_string(&args.spec) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("io error: {e}");
            return Err(3);
        }
    };
    let plan_src = match fs::read_to_string(&args.plan) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("io error: {e}");
            return Err(3);
        }
    };
    let spec_path = args.spec.display().to_string();
    let plan_path = args.plan.display().to_string();
    let output = match run_slice(&spec_src, &plan_src, &spec_path, &plan_path, &cfg) {
        Ok(o) => o,
        Err(e) => return exit_from_error(e),
    };
    if let Err(e) = fs::create_dir_all(&args.out) {
        eprintln!("io error: {e}");
        return Err(3);
    }
    for (name, body) in &output.bundles {
        let path = args.out.join(name);
        if let Err(e) = fs::write(&path, body) {
            eprintln!("io error: {e}");
            return Err(3);
        }
    }
    let manifest_json = serde_json::to_string_pretty(&output.manifest).map_err(|e| {
        eprintln!("io error: {e}");
        3
    })?;
    if let Err(e) = fs::write(args.out.join("manifest.json"), manifest_json) {
        eprintln!("io error: {e}");
        return Err(3);
    }
    if bind {
        if let Err(e) = fs::create_dir_all(".specometer") {
            eprintln!("io error: {e}");
            return Err(3);
        }
        let bound = serde_json::to_string_pretty(&output.manifest).map_err(|_| 3)?;
        if let Err(e) = fs::write(".specometer/manifest.json", bound) {
            eprintln!("io error: {e}");
            return Err(3);
        }
    }
    print_warning_summary(&output.manifest);
    Ok(())
}

fn execute_validate(args: &ValidateArgs) -> Result<(), i32> {
    let mut cfg = match config_from_args(&args.heuristic, args.strict) {
        Ok(c) => c,
        Err(e) => return exit_from_error(e),
    };
    cfg.strict_unknown_refs = args.strict;
    let spec_src = fs::read_to_string(&args.spec).map_err(|e| {
        eprintln!("io error: {e}");
        3
    })?;
    let plan_src = fs::read_to_string(&args.plan).map_err(|e| {
        eprintln!("io error: {e}");
        3
    })?;
  match validate_inputs(&spec_src, &plan_src, &cfg) {
        Ok(warnings) => {
            for w in &warnings {
                eprintln!("warning: {}", w.message);
            }
            if args.strict && !warnings.is_empty() {
                return Err(2);
            }
            Ok(())
        }
        Err(e) => exit_from_error(e),
    }
}

fn config_from_args(heuristic: &str, strict: bool) -> Result<ResolverConfig, SpecometerError> {
    let mode = match heuristic {
        "when-no-explicit" => HeuristicMode::WhenNoExplicit,
        "always" => HeuristicMode::Always,
        "never" => HeuristicMode::Never,
        other => {
            return Err(SpecometerError::Parse(format!(
                "unknown heuristic mode: {other}"
            )));
        }
    };
    Ok(ResolverConfig {
        heuristic: mode,
        strict_unknown_refs: strict,
        ..ResolverConfig::default()
    })
}

fn exit_from_error(e: SpecometerError) -> Result<(), i32> {
    match e {
        SpecometerError::Parse(msg) => {
            eprintln!("parse error: {msg}");
            Err(1)
        }
        SpecometerError::Validation(msg) => {
            eprintln!("validation failed: {msg}");
            Err(2)
        }
        SpecometerError::Io(err) => {
            eprintln!("io error: {err}");
            Err(3)
        }
    }
}

fn print_warning_summary(
    manifest: &specometer_core::bundle::Manifest,
) {
    for task in &manifest.tasks {
        if task.warnings.is_empty() {
            continue;
        }
        eprintln!("{}: {}", task.slug, task.warnings.join("; "));
    }
}
