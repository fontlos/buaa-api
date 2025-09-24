use std::path::{Path, PathBuf};

/// Join a path with the current executable's directory if it's a relative path
pub fn join_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(path)
    }
}
