use std::path::PathBuf;
mod cmd;
use clap::{ArgAction, Parser, Subcommand};
use rdocs::out;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(global = true, short, long, value_enum, default_value = "INFO")]
    /// Log level
    log_level: LevelFilter,

    /// Source code directory for collecting documentation
    #[clap(global = true, index = 1, default_value = ".")]
    path: PathBuf,

    /// Source code directory for collecting documentation
    #[arg(global = true, short, long, default_value = None)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Collect documentation blocks
    Collect {
        /// Save result to the given file. if not provided the results will
        /// print to the stdout .
        #[arg(short, long, default_value = None)]
        output: Option<PathBuf>,

        /// Result output
        #[arg(short, long, value_enum, default_value = None)]
        format: Option<out::Format>,
    },
    /// Collect documentation blocks and replace with a given target
    Replace {
        /// Location of replacement content. if empty take the default path
        #[clap(index = 2)]
        replace_path: Option<PathBuf>,

        /// Show the replacement operation without changes
        #[clap(long, action=ArgAction::SetTrue)]
        dry_run: bool,
    },
}

fn main() {
    let app: Cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(app.log_level.into())
                .from_env_lossy(),
        )
        .with_line_number(true)
        .with_target(true)
        .init();

    let span = tracing::span!(tracing::Level::TRACE, "parser");
    let _guard = span.enter();

    // println!("{:#?}", app.command.);
    match app.command {
        Commands::Collect { output, format } => {
            cmd::collect::exec(app.config.as_ref(), app.path.as_path(), format, output)
        }
        Commands::Replace {
            replace_path,
            dry_run,
        } => {
            let replace_path = replace_path.unwrap_or_else(|| app.path.clone());
            cmd::replace::exec(
                app.config.as_ref(),
                app.path.as_path(),
                replace_path.as_path(),
                dry_run,
            )
        }
    }
    .exit();
}
