pub use crate::prelude::*;

pub fn generate_sample_source() -> Result<()> {
    fn generate_frontmatter(date_str: &str) -> String {
        format!(
            r#"---
title: "Sample Markdown"
description: "This is a sample markdown file with frontmatter."
date: {date_str}
---"#
        )
    }

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

        let md_with_frontmatter = if file_path.ends_with(".md") {
            let date_str = "2025-07-29"; // Example date
            format!("{}\n\n{}", generate_frontmatter(date_str), md)
        } else {
            md.to_string()
        };

        std::fs::write(&full_path, md_with_frontmatter)?;
    }

    info!("Sample source files generated in: {}", root.display());

    Ok(())
}
