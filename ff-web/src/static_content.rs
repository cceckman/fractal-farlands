use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Result;

pub async fn get(Path(file): Path<String>) -> Result<&'static str> {
    match file.as_str() {
        "style.css" => Ok(include_str!("static/style.css")),
        _ => Err(StatusCode::NOT_FOUND.into()),
    }
}
