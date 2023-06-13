use include_dir::{include_dir, Dir};
use std::path::Path;
use warp::{path::FullPath, Rejection, Reply};

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../taxy-webui/dist");

pub async fn get(path: FullPath) -> Result<impl Reply, Rejection> {
    let path = path.as_str();
    if path.starts_with("/api/") {
        return Err(warp::reject::not_found());
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
    if let Some(file) = STATIC_DIR.get_file(path) {
        let ext = file
            .path()
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let mime = mime_guess::from_path(ext).first_or_octet_stream();
        Ok(warp::reply::with_header(
            warp::reply::with_header(file.contents(), "Content-Encoding", "gzip"),
            "Content-Type",
            mime.to_string(),
        ))
    } else {
        Err(warp::reject::not_found())
    }
}
