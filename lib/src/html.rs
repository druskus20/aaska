use crate::internal_prelude::*;

pub fn generate_html(input_mds: Vec<&str>) -> Result<Vec<String>> {
    let mut res = Vec::with_capacity(input_mds.len());
    for input_md in &input_mds {
        res.push(generate_html_single(input_md)?);
    }
    Ok(res)
}

pub fn generate_html_single(input_md: &str) -> Result<String> {
    let html = match markdown::to_html_with_options(input_md, &markdown::Options::gfm()) {
        Ok(html) => html,
        Err(msg) => {
            return Err(eyre!("Failed to convert markdown to HTML: {msg}"));
        }
    };

    Ok(html)
}
