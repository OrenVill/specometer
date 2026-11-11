use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "specometer", version = specometer_core::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version (stub until slice is implemented)
    Stub,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Stub => println!("{}", specometer_core::VERSION),
    }
    Ok(())
}
