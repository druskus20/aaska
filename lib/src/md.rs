use chrono::{DateTime, Utc};
use comrak::{
    Arena, ComrakOptions,
    nodes::{AstNode, NodeValue},
    parse_document,
};
use serde::Deserialize;

use crate::{fs::FileMeta, internal_prelude::*};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub meta: FileMeta,
    pub contents: FileContents,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FrontmatterData {
    pub title: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FileContents {
    pub frontmatter: Option<FrontmatterData>,
    pub body_ast: String, // Store the body content as string since comrak AST can't be easily cloned
}

pub fn parse_markdown(content: &str, options: &ComrakOptions) -> Result<FileContents> {
    // First, try to extract frontmatter if it exists
    let (frontmatter, body_content) = extract_frontmatter(content)?;

    // Parse the body content (without frontmatter) using comrak
    let arena = Arena::new();
    let root = parse_document(&arena, &body_content, options);

    // For now, we'll store the body content as string since working with comrak AST
    // requires lifetime management that's complex for this structure
    Ok(FileContents {
        frontmatter,
        body_ast: body_content,
    })
}

#[derive(Debug)]
pub struct PageList {
    pub files: Vec<ParsedFile>,
}

impl PageList {
    pub fn iter(&self) -> impl Iterator<Item = &ParsedFile> {
        self.files.iter()
    }

    pub fn sorted_by_date(&self) -> Vec<&ParsedFile> {
        let mut sorted_files: Vec<&ParsedFile> = self.files.iter().collect();
        sorted_files.sort_by(|a, b| {
            // First try to use the frontmatter date, if it exists
            let date_a = a.contents.frontmatter.as_ref().and_then(|fm| fm.date);
            let date_b = b.contents.frontmatter.as_ref().and_then(|fm| fm.date);

            // default to the file's date if frontmatter date is not available
            let date_a = date_a.unwrap_or(a.meta.date);
            let date_b = date_b.unwrap_or(b.meta.date);

            date_a.cmp(&date_b)
        });

        sorted_files
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

            // Parse frontmatter as YAML
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
    use super::*;

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

        let options = ComrakOptions::default();
        let parsed = parse_markdown(markdown, &options).expect("Should parse without error");

        assert!(parsed.frontmatter.is_some());

        let frontmatter = parsed.frontmatter.unwrap();
        assert_eq!(frontmatter.title, Some("Hello World".to_string()));
        assert_eq!(
            frontmatter.date,
            Some("2025-07-29T00:00:00Z".parse().unwrap())
        );
        assert_eq!(
            frontmatter.tags,
            Some(vec!["rust".to_string(), "markdown".to_string()])
        );

        // Check that the body content doesn't include frontmatter
        assert!(parsed.body_ast.contains("# Heading"));
        assert!(!parsed.body_ast.contains("---"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let markdown = r#"# Just a heading

Some body content here without frontmatter.
"#;

        let options = ComrakOptions::default();
        let parsed = parse_markdown(markdown, &options).expect("Should parse without error");

        assert!(parsed.frontmatter.is_none());
        assert!(parsed.body_ast.contains("# Just a heading"));
    }
}
