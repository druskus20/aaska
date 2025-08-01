use std::path::PathBuf;

use argus::tracing::TracingOptions;
use clap::{Parser, Subcommand};
use tracing::Level;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct RawArgs {
    #[clap(subcommand)]
    command: RawCommand,
    #[clap(short, long, default_value = "false")]
    pretty_print: bool,
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    verbosity: u8,
    #[arg(long, global = true, default_value = "false")]
    no_color: bool,
}

#[derive(Subcommand, Clone, Debug)]
pub enum RawCommand {
    Sample,
    Generate {
        #[arg(short, long)]
        input: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub(crate) struct ParsedArgs {
    pub command: Command,
    pub tracing_options: TracingOptions,
}

impl ParsedArgs {
    pub fn parse_raw() -> Self {
        let args: RawArgs = clap::Parser::parse();

        let log_level = match args.verbosity {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        };

        let command = match args.command {
            RawCommand::Sample => Command::Sample,
            RawCommand::Generate { input, output } => {
                Command::Generate(GenerateArgs { input, output })
            }
        };

        ParsedArgs {
            command,
            tracing_options: TracingOptions {
                log_level,
                pretty_print: args.pretty_print,
                color: !args.no_color,
                lines: true,
                file: true,
                output: argus::tracing::Output::Stdout,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Sample,
    Generate(GenerateArgs),
}

#[derive(Debug)]
pub struct GenerateArgs {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
}
