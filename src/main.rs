#[allow(unused_imports)]
mod prelude {
    pub use color_eyre::eyre::{Result, WrapErr};
    pub use tracing::{debug, error, info, instrument, span, trace, warn};
}
use aaska::comrak::ComrakOptions;
use prelude::*;
use std::path::PathBuf;

mod cli;
mod index;

mod cmds;

struct Config<'c> {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub comrak_options: ComrakOptions<'c>,
}

pub struct SiteMetadata {
    pub author: &'static str,
}

fn validate_config(config: &Config) -> Result<()> {
    if !config.output_dir.exists() {
        std::fs::create_dir_all(&config.output_dir)
            .wrap_err("Failed to create output directory")?;
    } else if !config.output_dir.is_dir() {
        return Err(color_eyre::eyre::eyre!(
            "Output path is not a directory: {}",
            config.output_dir.display()
        ));
    }
    Ok(())
}

fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    let args = cli::ParsedArgs::parse_raw();
    let _guard = argus::tracing::setup_tracing(&args.tracing_options);

    match args.command {
        cli::Command::Generate(args) => cmds::generate::generate(args),
        cli::Command::Sample => cmds::sample::generate_sample_source(),
    }
    .expect("Failed to execute command");
}
