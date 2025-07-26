use crate::internal_prelude::*;
use std::path::Path;

pub fn generate_html_from_md(input_md: &str) -> Result<String> {
    let html = match markdown::to_html_with_options(input_md, &markdown::Options::gfm()) {
        Ok(html) => html,
        Err(msg) => {
            return Err(eyre!("Failed to convert markdown to HTML: {msg}"));
        }
    };

    Ok(html)
}
