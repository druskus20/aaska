use chrono::{DateTime, Utc};
use markdown::{
    ParseOptions,
    mdast::{Node, Yaml},
};
use serde::Deserialize;

use crate::internal_prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentFile {
    pub path: PathBuf,
    pub date: DateTime<Utc>,
    pub file_type: FileType,
    pub maybe_frontmatter: Option<FrontmatterData>,
}

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

pub fn parse_markdown_frontmatter(
    content: &str,
    parse_options: &ParseOptions,
) -> Result<Option<FrontmatterData>> {
    let ast = markdown::to_mdast(content, parse_options)
        .map_err(|e| eyre!("Failed to parse markdown content: {}", e))?;
    dbg!(&ast);

    let mut frontmatter_yaml = None;

    if let Node::Root(root) = ast {
        for node in root.children {
            if let Node::Yaml(Yaml { value, .. }) = node {
                frontmatter_yaml = Some(value);
                break;
            }
        }
    };

    if let Some(yaml) = frontmatter_yaml {
        let frontmatter: FrontmatterData = serde_yaml::from_str(&yaml)
            .map_err(|e| eyre!("Failed to parse frontmatter YAML: {}", e))?;
        Ok(Some(frontmatter))
    } else {
        Ok(None)
    }
}

pub fn list_files_dir(
    dir: &impl AsRef<Path>,
    parse_options: &ParseOptions,
) -> Result<Vec<ContentFile>> {
    utils::assert_dir_exists(dir);

    let dir = dir.as_ref();
    let mut res = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            res.push(utils::get_file_data(&path, parse_options)?);
        }
    }

    Ok(res)
}

// traverse a directory recursively and list all files
pub fn list_files_dir_rec(
    dir: &impl AsRef<Path>,
    parse_options: &ParseOptions,
) -> Result<PageList> {
    utils::assert_dir_exists(dir);

    let mut dis = vec![dir.as_ref().to_path_buf()];
    let mut files = Vec::new();
    while let Some(current_dir) = dis.pop() {
        for entry in current_dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(utils::get_file_data(&path, parse_options)?);
            } else if path.is_dir() {
                dis.push(path.canonicalize()?);
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
    pub fn sorted_by_date(&self) -> Vec<&ContentFile> {
        // sort by page.maybe_frontmatter.date
        // or by file date

        let mut sorted_files: Vec<&ContentFile> = self.files.iter().collect();
        sorted_files.sort_by(|a, b| {
            // First try to use the frontmatter date, if it exists
            let date_a = a.maybe_frontmatter.as_ref().and_then(|fm| fm.date);
            let date_b = b.maybe_frontmatter.as_ref().and_then(|fm| fm.date);

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

    pub fn get_file_data(
        path: &impl AsRef<Path>,
        parse_options: &ParseOptions,
    ) -> Result<ContentFile> {
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

        let maybe_frontmatter = if file_type == FileType::Markdown {
            let content = std::fs::read_to_string(path)?;
            parse_markdown_frontmatter(&content, parse_options)?
        } else {
            None
        };

        Ok(ContentFile {
            path: path.to_path_buf().canonicalize()?,
            date,
            file_type,
            maybe_frontmatter,
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
    use markdown::Constructs;

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

        let opts = ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let parsed =
            parse_markdown_frontmatter(markdown, &opts).expect("Should parse without error");

        assert!(parsed.is_some());

        let frontmatter = parsed.unwrap();
        assert_eq!(frontmatter.title, Some("Hello World".to_string()));
        assert_eq!(frontmatter.date, Some("2025-07-29".parse().unwrap()));
        assert_eq!(
            frontmatter.tags,
            Some(vec!["rust".to_string(), "markdown".to_string()])
        );
    }
}
