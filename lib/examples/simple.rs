use std::path::PathBuf;

use color_eyre::eyre::Result;

fn main() {
    let root = create_tmp_files().expect("Failed to create temporary files");
    let files = aaska_lib::fs::list_files_dir_rec(&root).expect("Failed to list files");

    let files = files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "md"))
        .collect::<Vec<&PathBuf>>();

    for file in files {
        let html = aaska_lib::markdown::generate_html_single(
            &std::fs::read_to_string(file).expect("Failed to read markdown file"),
        )
        .expect("Failed to generate HTML from markdown");
        println!("HTML for {}:\n{}", file.display(), html);
    }
}

fn create_tmp_files() -> Result<PathBuf> {
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

    println!("Temporary markdown files created under: {}", root.display());

    Ok(root)
}
