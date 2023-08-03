use fnv::FnvHasher;
use hyper::StatusCode;
use include_dir::{include_dir, Dir};
use std::hash::Hasher;
use std::path::Path;
use warp::{path::FullPath, reply::WithStatus, Rejection, Reply};

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dist");

const IMMUTABLE_FILE_PREFIXES: &[&str] = &["taxy-webui-", "tailwind-"];

pub async fn get(path: FullPath, if_none_match: Option<String>) -> Result<impl Reply, Rejection> {
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

    let immutable = IMMUTABLE_FILE_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix));

    let cache_control = if immutable {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=604800, must-revalidate"
    };

    let path = Path::new("webui").join(format!("{path}.gz"));
    if let Some(file) = STATIC_DIR.get_file(path) {
        let mut hasher = FnvHasher::default();
        hasher.write(file.contents());
        let etag = format!("{:x}", hasher.finish());

        let ext = file
            .path()
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let mime = mime_guess::from_path(ext).first_or_octet_stream();
        let reply: WithStatus<&[u8]> = if if_none_match.as_ref() == Some(&etag) {
            warp::reply::with_status(&[], StatusCode::NOT_MODIFIED)
        } else {
            warp::reply::with_status(file.contents(), StatusCode::OK)
        };
        let reply = warp::reply::with_header(reply, "Content-Encoding", "gzip");
        let reply = warp::reply::with_header(reply, "Content-Type", mime.to_string());
        let reply = warp::reply::with_header(reply, "Cache-Control", cache_control);
        let reply = warp::reply::with_header(reply, "ETag", etag);
        Ok(reply)
    } else {
        Err(warp::reject::not_found())
    }
}
