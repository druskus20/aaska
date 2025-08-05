use std::path::PathBuf;

use comrak::ComrakOptions;

use crate::{
    fs::FileMeta,
    md::{Html, ParsedFile},
};

#[derive(Debug)]
pub struct GeneratedFileMeta {
    pub title: String,
    pub description: String,
}

#[derive(Debug)]
pub struct GeneratedFile {
    pub contents: Html,
    pub original_md_path: PathBuf,
    pub meta: GeneratedFileMeta,
}
pub fn generate_html(file: &ParsedFile, options: &ComrakOptions) -> GeneratedFile {
    let mut out = vec![];
    comrak::format_html(file.contents.body_ast, options, &mut out)
        .expect("Failed to format HTML from Markdown");

    dbg!(GeneratedFile {
        contents: Html(String::from_utf8_lossy(&out).to_string()),
        original_md_path: file.meta.path.clone(),
        meta: GeneratedFileMeta {
            title: "title".into(),
            description: "desc".into(),
        },
    })
}
