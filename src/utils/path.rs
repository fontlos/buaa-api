use std::path::{Path, PathBuf};

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