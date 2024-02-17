use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Result};

pub async fn get(Path(file): Path<String>) -> Result<impl IntoResponse> {
    match file.as_str() {
        "style.css" => Ok(get_style()),
        _ => Err(StatusCode::NOT_FOUND.into()),
    }
}

fn get_style() -> impl IntoResponse {
    const STYLE: &'static str = include_str!("static/style.css");

    let headers = [
        (header::CONTENT_TYPE, "text/css")
    ];
    (headers, STYLE)
}
