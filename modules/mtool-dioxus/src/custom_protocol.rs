use std::path::{Path, PathBuf};

use bevy::log::warn;
use dioxus_desktop::{
    wry::{
        http::{Request, Response},
        WebViewId,
    },
    RequestAsyncResponder,
};
use mapp::{anyhow, tokio, tracing::info};

fn get_mime_from_path(asset: &Path) -> Result<&'static str, anyhow::Error> {
    if asset.extension().is_some_and(|ext| ext == "svg") {
        return Ok("image/svg+xml");
    }

    match infer::get_from_path(asset)?.map(|f| f.mime_type()) {
        Some(f) if f != "text/plain" => Ok(f),
        _other => Ok(get_mime_by_ext(asset)),
    }
}

fn get_mime_by_ext(trimmed: &Path) -> &'static str {
    match trimmed.extension().and_then(|e| e.to_str()) {
        // The common assets are all utf-8 encoded
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",

        // the rest... idk? probably not
        Some("mjs") => "text/javascript; charset=utf-8",
        Some("bin") => "application/octet-stream",
        Some("csv") => "text/csv",
        Some("ico") => "image/vnd.microsoft.icon",
        Some("jsonld") => "application/ld+json",
        Some("rtf") => "application/rtf",
        Some("mp4") => "video/mp4",
        // Assume HTML when a TLD is found for eg. `dioxus:://dioxuslabs.app` | `dioxus://hello.com`
        Some(_) => "text/html; charset=utf-8",
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
        // using octet stream according to this:
        None => "application/octet-stream",
    }
}

async fn send_response(
    uri_path: PathBuf,
    responder: RequestAsyncResponder,
) -> Result<(), anyhow::Error> {
    let mime_type = get_mime_from_path(&uri_path);

    responder.respond(
        Response::builder()
            .header("Content-Type", mime_type?)
            .header("Access-Control-Allow-Origin", "*")
            .body(tokio::fs::read(uri_path).await?)?,
    );

    Ok(())
}

pub fn file_handler(_: WebViewId, request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let uri_path = PathBuf::from(
        urlencoding::decode(request.uri().path().trim_matches('/'))
            .expect("expected URL to be UTF-8 encoded")
            .as_ref(),
    );

    if uri_path.exists() {
        tokio::spawn(async move {
            if let Err(e) = send_response(uri_path, responder).await {
                warn!("{:?}", e);
            }
        });
    }
}
