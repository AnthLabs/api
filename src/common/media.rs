use std::path::PathBuf;

pub fn media_root() -> PathBuf {
    std::env::var("MEDIA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("media"))
}

pub fn uploads_directory() -> PathBuf {
    media_root().join("uploads")
}

pub fn hls_directory() -> PathBuf {
    media_root().join("hls")
}
