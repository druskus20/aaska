use std::path::PathBuf;

use aaska::comrak::{ComrakOptions, ExtensionOptions};

pub use crate::prelude::*;

pub fn generate(args: crate::cli::GenerateArgs) -> Result<()> {
    let config = crate::Config {
        source_dir: args.input.unwrap_or_else(|| PathBuf::from("/tmp/input")),
        output_dir: args.output.unwrap_or_else(|| PathBuf::from("/tmp/output")),
        comrak_options: ComrakOptions::default(),
    };

    crate::validate_config(&config).expect("Configuration validation failed");

    let meta = crate::SiteMetadata { author: "druskus" };
    let post_list =
        aaska::fs::list_files_dir_rec(&config.source_dir).expect("Failed to list source directory");

    let opts = ComrakOptions {
        extension: ExtensionOptions {
            front_matter_delimiter: Some("---".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let arena = aaska::comrak::Arena::new();
    let parser = aaska::md::MarkdownParser::with_arena(&arena, &opts);
    let parsed = parser.parse_many(&post_list)?;

    for file in &parsed {
        let dest_path = config
            .output_dir
            .join(format!("{}.html", file.meta.file_name_no_stem()));

        let generated_file = aaska::html::generate_html(file, &config.comrak_options);

        std::fs::write(dest_path, generated_file.contents.0).wrap_err_with(|| {
            format!(
                "Failed to write HTML for file: {}",
                file.meta.path.display()
            )
        })?;
    }

    let index = crate::index::index_html(meta, &parsed.into());
    std::fs::write(config.output_dir.join("index.html"), index)?;

    info!(
        "Site generated successfully at: {}",
        config.output_dir.display()
    );

    Ok(())
}
