//! Utility functions and modules

mod path;
pub use path::join_dir;

mod str;
pub(crate) use str::{gen_rand_str, parse_by_tag};

mod time;
pub use time::*;
