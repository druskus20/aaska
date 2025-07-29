use std::path::PathBuf;

#[allow(unused_imports)]
mod prelude {
    pub use color_eyre::eyre::{Result, WrapErr};
    pub use tracing::{debug, error, info, instrument, span, trace, warn};
}
use prelude::*;

mod cli;
mod index;

struct Config {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
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

fn generate_sample_source() -> Result<()> {
    let md = r#"
# Hello, World!

This is a simple markdown example.

```rust
fn main() {
    println!("Hello, World!");
}
```

This is a code block in Rust.
    "#;

    // Create root temp dir: /tmp/aaska/<pid>
    let root = std::env::temp_dir()
        .join("aaska")
        .join(std::process::id().to_string());

    // Define nested structure
    let structure = vec![
        ("example.md", &root),
        ("pages/intro.md", &root),
        ("pages/chapters/ch1.md", &root),
        ("pages/chapters/ch2.md", &root),
        ("pages2/woop/README.md", &root),
    ];
    for (file_path, base_dir) in structure {
        let full_path = base_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, md)?;
    }

    info!("Sample source files generated in: {}", root.display());
    Ok(())
}

fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    let args = cli::ParsedArgs::parse_raw();
    let _guard = argus::tracing::setup_tracing(&args.tracing_options);

    match args.command {
        cli::Command::Generate(args) => generate(args),
        cli::Command::Sample => generate_sample_source(),
    }
    .expect("Failed to execute command");
}

fn generate(args: cli::GenerateArgs) -> Result<()> {
    let config = Config {
        source_dir: args.input.unwrap_or_else(|| PathBuf::from("/tmp/input")),
        output_dir: args.output.unwrap_or_else(|| PathBuf::from("/tmp/output")),
    };

    let meta = SiteMetadata { author: "druskus" };

    validate_config(&config).expect("Configuration validation failed");
    info!("Configuration validated successfully");

    let post_list =
        aaska::fs::list_files_dir_rec(&config.source_dir).expect("Failed to list source directory");

    dbg!(post_list);

    let index = index::index_html(meta);

    Ok(())
}
