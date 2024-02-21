use axum::extract::Path;
use axum::http::{header, HeaderName, StatusCode};
use axum::response::{IntoResponse, Result};

pub async fn get(Path(file): Path<String>) -> Result<impl IntoResponse> {
    match file.as_str() {
        "style.css" => Ok(get_style()),
        "app.js" => Ok(get_app()),
        _ => Err(StatusCode::NOT_FOUND.into()),
    }
}


type StaticHeaders = [(HeaderName, &'static str); 1];

const fn headers(content_type: &'static str) -> StaticHeaders {
    [
        (header::CONTENT_TYPE, content_type)
    ]
}

type StaticResponse = (StaticHeaders, &'static str);

fn get_style() -> StaticResponse {
    const STYLE: &'static str = include_str!("static/style.css");
    (headers("text/css"), STYLE)
}

fn get_app() -> StaticResponse {
    const APP: &'static str = include_str!("static/app.js");
    (headers("text/javascript"), APP)
}
