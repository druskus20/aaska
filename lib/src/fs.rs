use chrono::{DateTime, Utc};
use comrak::{
    Arena, ComrakOptions,
    nodes::{AstNode, NodeValue},
    parse_document,
};
use serde::Deserialize;

use crate::internal_prelude::*;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentFile {
    pub path: PathBuf,
    pub date: DateTime<Utc>,
    pub file_type: FileType,
    pub contents: FileContents,
}

#[derive(Debug, Clone)]
pub struct FileContents {
    pub frontmatter: Option<FrontmatterData>,
    pub body_ast: String, // Store the body content as string since comrak AST can't be easily cloned
}

// Manual PartialEq implementation since AstNode doesn't implement it
impl PartialEq for FileContents {
    fn eq(&self, other: &Self) -> bool {
        self.frontmatter == other.frontmatter && self.body_ast == other.body_ast
    }
}

impl Eq for FileContents {}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FrontmatterData {
    pub title: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    HTML,
    Markdown,
    PlainText,
    Unsupported,
}

impl From<PathBuf> for FileType {
    fn from(path: PathBuf) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => FileType::HTML,
            Some("md") => FileType::Markdown,
            Some("txt") => FileType::PlainText,
            _ => FileType::Unsupported,
        }
    }
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

pub fn parse_files_dir(
    dir: &impl AsRef<Path>,
    options: &ComrakOptions,
) -> Result<Vec<ContentFile>> {
    utils::assert_dir_exists(dir);

    let dir = dir.as_ref();
    let mut res = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            res.push(utils::get_file_data(&path, options)?);
        }
    }

    Ok(res)
}

// traverse a directory recursively and list all files
pub fn parse_files_dir_rec(dir: &impl AsRef<Path>, options: &ComrakOptions) -> Result<PageList> {
    utils::assert_dir_exists(dir);

    let mut dirs = vec![dir.as_ref().to_path_buf()];
    let mut files = Vec::new();
    while let Some(current_dir) = dirs.pop() {
        for entry in current_dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(utils::get_file_data(&path, options)?);
            } else if path.is_dir() {
                dirs.push(path.canonicalize()?);
            }
        }
    }

    Ok(PageList { files })
}

#[derive(Debug)]
pub struct PageList {
    pub files: Vec<ContentFile>,
}

impl PageList {
    pub fn iter(&self) -> impl Iterator<Item = &ContentFile> {
        self.files.iter()
    }

    pub fn sorted_by_date(&self) -> Vec<&ContentFile> {
        let mut sorted_files: Vec<&ContentFile> = self.files.iter().collect();
        sorted_files.sort_by(|a, b| {
            // First try to use the frontmatter date, if it exists
            let date_a = a.contents.frontmatter.as_ref().and_then(|fm| fm.date);
            let date_b = b.contents.frontmatter.as_ref().and_then(|fm| fm.date);

            // default to the file's date if frontmatter date is not available
            let date_a = date_a.unwrap_or(a.date);
            let date_b = date_b.unwrap_or(b.date);

            date_a.cmp(&date_b)
        });

        sorted_files
    }
}

mod utils {
    use super::*;

    pub fn get_file_data(path: &impl AsRef<Path>, options: &ComrakOptions) -> Result<ContentFile> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(eyre!("Path does not exist: {:?}", path));
        }

        let file_type = FileType::from(path.to_path_buf());
        let date: DateTime<Utc> = path
            .metadata()?
            .modified()
            .wrap_err(format!("Failed to get modified date for file: {path:?}"))?
            .into();

        let contents = if file_type == FileType::Markdown {
            let content = std::fs::read_to_string(path)?;
            parse_markdown(&content, options)?
        } else {
            FileContents {
                frontmatter: None,
                body_ast: std::fs::read_to_string(path).unwrap_or_default(),
            }
        };

        Ok(ContentFile {
            path: path.to_path_buf().canonicalize()?,
            date,
            file_type,
            contents,
        })
    }

    pub fn _assert_file_exists(path: &impl AsRef<Path>) {
        let path = path.as_ref();
        if !path.exists() {
            panic!("Path does not exist: {path:?}");
        }
        if !path.is_file() {
            panic!("Expected a file, but found a directory: {path:?}");
        }
    }

    pub fn assert_dir_exists(path: &impl AsRef<Path>) {
        let path = path.as_ref();
        if !path.exists() {
            panic!("Path does not exist: {path:?}");
        }
        if !path.is_dir() {
            panic!("Expected a directory, but found a file: {path:?}");
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
