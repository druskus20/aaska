fn main() {
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

    let html = aaska_lib::markdown::generate_html_from_md(md)
        .expect("Failed to generate HTML from markdown");

    println!("{html}");
}
