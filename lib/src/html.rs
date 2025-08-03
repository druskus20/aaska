use comrak::ComrakOptions;

use crate::{
    fs::FileMeta,
    md::{Html, ParsedFile},
};

pub struct GeneratedFileMeta {
    pub title: String,
    pub description: String,
}

pub struct GeneratedFile {
    pub contents: Html,
    pub path: String,
    pub meta: GeneratedFileMeta,
}
pub fn generate_html(
    file: &ParsedFile,
    out: &mut Vec<u8>,
    options: &ComrakOptions,
) -> GeneratedFile {
    comrak::format_html(file.contents.body_ast, options, out)
        .expect("Failed to format HTML from Markdown");

    GeneratedFile {
        contents: Html(String::from_utf8_lossy(out).to_string()),
    }
}
