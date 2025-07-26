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
//!     - Markdown: the simple way is to generate HTML directly. The complex way is to use the syntax tree and generate custom components.

pub mod internal_prelude {

    pub use color_eyre::eyre::{WrapErr, eyre};
    pub use color_eyre::{Context, Result};
    pub use tracing::{debug, error, info, trace, warn};
}

pub mod markdown;
