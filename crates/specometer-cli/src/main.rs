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
    /// Install CLI (run from repo root); use scripts/install.sh for agent skill menu
    Install(InstallArgs),
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
struct InstallArgs {
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    #[arg(long)]
    skills_only: bool,
    #[arg(long)]
    bin_only: bool,
    #[arg(long)]
    prefix: Option<PathBuf>,
    #[arg(long)]
    non_interactive: bool,
    #[arg(long)]
    all_agents: bool,
    #[arg(long)]
    agents: Option<String>,
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
        Commands::Install(args) => {
            execute_install(&args)?;
        }
    }
    Ok(())
}

fn execute_install(args: &InstallArgs) -> Result<(), i32> {
    let repo = fs::canonicalize(&args.repo).map_err(|e| {
        eprintln!("io error: {e}");
        3
    })?;
    let script = repo.join("scripts/install.sh");
    if !script.is_file() {
        eprintln!(
            "error: {} not found; clone https://github.com/orenvill/specometer or pass --repo",
            script.display()
        );
        return Err(1);
    }
    let mut cmd = std::process::Command::new("bash");
    cmd.arg(&script);
    if args.skills_only {
        cmd.arg("--skills-only");
    }
    if args.bin_only {
        cmd.arg("--bin-only");
    }
    if args.non_interactive {
        cmd.arg("--non-interactive");
    }
    if args.all_agents {
        cmd.arg("--all-agents");
    }
    if let Some(ref agents) = args.agents {
        cmd.arg("--agents").arg(agents);
    }
    let prefix = args
        .prefix
        .clone()
        .or_else(|| std::env::var("PREFIX").ok().map(PathBuf::from));
    if let Some(prefix) = prefix {
        cmd.arg("--prefix").arg(prefix);
    }
    let status = cmd.status().map_err(|e| {
        eprintln!("io error: {e}");
        3
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(status.code().unwrap_or(1))
    }
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
