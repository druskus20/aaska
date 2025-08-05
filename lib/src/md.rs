use chrono::{DateTime, NaiveDate, Utc};
use comrak::{
    Arena, ComrakOptions, ExtensionOptions, arena_tree::Node, nodes::Ast, parse_document,
};
use serde::Deserialize;

use crate::{
    fs::{FileMeta, FileType},
    html::GeneratedFile,
    internal_prelude::*,
};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFileMeta {
    pub path: PathBuf,
    pub date: DateTime<Utc>,
    pub file_type: FileType,
}

impl ParsedFileMeta {
    pub fn file_name_no_stem(&self) -> &str {
        self.path.file_name().and_then(|s| s.to_str()).unwrap_or("")
    }

    pub fn stem(&self) -> &str {
        self.path.file_stem().and_then(|s| s.to_str()).unwrap_or("")
    }

    pub fn file_name(&self) -> &str {
        self.path.file_name().and_then(|s| s.to_str()).unwrap_or("")
    }
}

#[derive(Debug, Clone)]
pub struct ParsedFile<'c> {
    pub meta: ParsedFileMeta,
    pub contents: FileContents<'c>,
}

impl<'a> ParsedFile<'a> {
    fn new(meta: &'a FileMeta, contents: FileContents<'a>) -> ParsedFile<'a> {
        ParsedFile {
            meta: ParsedFileMeta {
                path: meta.path.clone(),
                date: meta.date,
                file_type: meta.file_type,
            },
            contents,
        }
    }

    fn to_html(&self, options: &ComrakOptions) -> Result<GeneratedFile> {
        Ok(crate::html::generate_html(&self, options))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FrontmatterData {
    pub title: Option<String>,
    pub date: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FileContents<'a> {
    pub frontmatter: Option<FrontmatterData>,
    pub body_ast: &'a Node<'a, RefCell<Ast>>,
}

pub struct MarkdownParser<'a, 'c> {
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    options: &'c ComrakOptions<'c>,
}

impl<'a, 'c> MarkdownParser<'a, 'c> {
    pub fn with_arena(
        arena: &'a Arena<Node<'a, RefCell<Ast>>>,
        options: &'c ComrakOptions,
    ) -> Self {
        MarkdownParser { arena, options }
    }

    pub fn parse_markdown(&self, content: &str) -> Result<FileContents<'a>> {
        // First, try to extract frontmatter if it exists
        let (frontmatter, body_content) = extract_frontmatter(content)?;

        let root: &'a Node<'a, RefCell<Ast>> =
            parse_document(self.arena, &body_content, &self.options);

        Ok(FileContents {
            frontmatter,
            body_ast: root,
        })
    }

    pub fn parse_many(&self, files: &'a Vec<FileMeta>) -> Result<Vec<ParsedFile<'a>>> {
        let mut acc = vec![];
        for f in files {
            let content = crate::fs::read_file(&f.path)?;
            let parsed_content = self.parse_markdown(&content)?;

            acc.push(ParsedFile::new(f, parsed_content));
        }

        Ok(acc)
    }
}

#[derive(Debug)]
pub struct PageList<'c> {
    pub files: Vec<ParsedFile<'c>>,
}

impl<'a> From<Vec<ParsedFile<'a>>> for PageList<'a> {
    fn from(files: Vec<ParsedFile<'a>>) -> Self {
        PageList { files }
    }
}

impl<'c> PageList<'c> {
    pub fn iter(&'c self) -> impl Iterator<Item = &'c ParsedFile<'c>> {
        self.files.iter()
    }

    pub fn sorted_by_date(&'c self) -> Vec<&'c ParsedFile<'c>> {
        let mut sorted_files: Vec<&ParsedFile> = self.files.iter().collect();
        sorted_files.sort_by(|a, b| {
            // First try to use the frontmatter date, if it exists
            let date_a = a.contents.frontmatter.as_ref().and_then(|fm| fm.date);
            let date_b = b.contents.frontmatter.as_ref().and_then(|fm| fm.date);

            // default to the file's date if frontmatter date is not available
            let date_a = date_a
                .and_then(|d| {
                    d.and_hms_opt(0, 0, 0)
                        .map(|nd| DateTime::from_naive_utc_and_offset(nd, Utc))
                })
                .unwrap_or(a.meta.date);

            let date_b = date_b
                .and_then(|d| {
                    d.and_hms_opt(0, 0, 0)
                        .map(|nd| DateTime::from_naive_utc_and_offset(nd, Utc))
                })
                .unwrap_or(b.meta.date);

            date_a.cmp(&date_b)
        });

        sorted_files
    }
}

#[derive(Debug)]
pub struct Html(pub String);

impl Deref for Html {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Html {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn extract_frontmatter(content: &str) -> Result<(Option<FrontmatterData>, String)> {
    let content = content.trim();

    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---") {
        return Ok((None, content.to_string()));
    }

    // Find the closing delimiter
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Ok((None, content.to_string()));
    }

    // Look for the closing --- after the first line
    let mut end_index = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_index = Some(i);
            break;
        }
    }

    match end_index {
        Some(end) => {
            // Extract frontmatter content (excluding the --- delimiters)
            let frontmatter_lines = &lines[1..end];
            let frontmatter_content = frontmatter_lines.join("\n");

            let frontmatter: FrontmatterData = serde_yaml::from_str(&frontmatter_content)
                .map_err(|e| eyre!("Failed to parse frontmatter as YAML: {}", e))?;

            // Get the body content (everything after the closing ---)
            let body_lines = &lines[end + 1..];
            let body_content = body_lines.join("\n");

            Ok((Some(frontmatter), body_content))
        }
        None => {
            // No closing delimiter found, treat entire content as body
            Ok((None, content.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::LazyCell;

    use super::*;

    fn default_opts<'c>() -> ComrakOptions<'c> {
        ComrakOptions {
            extension: ExtensionOptions {
                front_matter_delimiter: Some("---".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_parse_frontmatter() {
        let markdown = r#"---
title: "Hello World"
date: "2025-07-29"
tags:
  - rust
  - markdown
---

# Heading

Some body content here.
"#;

        let arena = Arena::new();
        let opts = default_opts();
        let parser = MarkdownParser::with_arena(&arena, &opts);
        let parsed = parser
            .parse_markdown(markdown)
            .expect("Should parse without error");

        assert!(parsed.frontmatter.is_some());

        let frontmatter = parsed.frontmatter.unwrap();
        assert_eq!(frontmatter.title, Some("Hello World".to_string()));
        assert_eq!(frontmatter.date, Some("2025-07-29".parse().unwrap()));
        assert_eq!(
            frontmatter.tags,
            Some(vec!["rust".to_string(), "markdown".to_string()])
        );
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let markdown = r#"# Just a heading

Some body content here without frontmatter.
"#;

        let arena = Arena::new();
        let opts = default_opts();
        let parser = MarkdownParser::with_arena(&arena, &opts);
        let parsed = parser
            .parse_markdown(markdown)
            .expect("Should parse without error");

        assert!(parsed.frontmatter.is_none());
    }
}
