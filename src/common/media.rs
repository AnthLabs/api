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

pub fn logs_directory() -> PathBuf {
    media_root().join("log")
}

pub fn keys_directory() -> PathBuf {
    std::env::var("HLS_KEY_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| media_root().join("keys"))
}