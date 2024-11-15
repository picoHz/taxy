use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode, Uri},
    response::IntoResponse,
};
use fnv::FnvHasher;
use include_dir::{include_dir, Dir};
use std::{hash::Hasher, path::Path};

use super::AppError;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dist");

const IMMUTABLE_FILE_PREFIXES: &[&str] = &["taxy-webui-", "tailwind-"];

pub async fn fallback(uri: Uri, req_headers: HeaderMap) -> Result<impl IntoResponse, AppError> {
    let mut headers = HeaderMap::new();

    let path = uri.path();
    if path.starts_with("/api/") {
        return Err(AppError::NotFound);
    }
    let path_has_extension = path
        .rfind('.')
        .map(|i| i > path.rfind('/').unwrap_or(0))
        .unwrap_or_default();
    let path = if path == "/" || !path_has_extension {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    let path = Path::new("webui").join(format!("{path}.gz"));

    let immutable = IMMUTABLE_FILE_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix));

    let cache_control = if immutable {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=604800, must-revalidate"
    };

    headers.insert("Content-Encoding", "gzip".parse().unwrap());
    headers.insert("Cache-Control", cache_control.parse().unwrap());

    if let Some(file) = STATIC_DIR.get_file(path) {
        let mut hasher = FnvHasher::default();
        hasher.write(file.contents());
        let etag = format!("{:x}", hasher.finish());
        headers.insert("ETag", etag.parse().unwrap());

        if req_headers
            .get("If-None-Match")
            .map(|x| x.to_str().unwrap_or_default())
            == Some(&etag)
        {
            return Ok((StatusCode::NOT_MODIFIED, headers, Bytes::new()));
        }

        let ext = file
            .path()
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let mime = mime_guess::from_path(ext).first_or_octet_stream();
        headers.insert("Content-Type", mime.to_string().parse().unwrap());

        return Ok((StatusCode::OK, headers, Bytes::from_static(file.contents())));
    }

    Err(AppError::NotFound)
}
