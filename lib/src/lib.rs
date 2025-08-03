//! The idea behind this lib.
//!
//! - Be simple - but extensible.
//! - Generate HTML from mardkwown.
//! - Allow me to hook into the HTML generation process
//! - Serve as a "custom pipeline" for generating my website.
//!
//!
//! What about styling?
//! - Maybe I just give up on compoents? - seems overly complex maybe
//! - If I do components:
//!     - How to handle sytle inheritance.
//!     - Components: how to do slots.
//!     - Markdown: the simple way is to generate HTML directly. The complex way is to use the
//!       syntax tree and generate custom components.
//!       
//!       
//! Phases:
//!     1 - fs_list: scan source files into file metadata -> metadata of the file
//!     2 - md: read files, parse into md -> generate metadata based on the frontmatter
//!     3 - html: generate HTML -> metadata at this stage contains relative paths to the files
//!

use comrak::ComrakOptions;
use std::path::PathBuf;

#[allow(unused_imports)]
mod internal_prelude {
    pub use color_eyre::eyre::{WrapErr, eyre};
    pub use color_eyre::{Context, Result};
    pub use tracing::{debug, error, info, trace, warn};
}

pub mod comrak {
    pub use comrak::*;
}

pub mod fs;
pub mod html;
pub mod md;
