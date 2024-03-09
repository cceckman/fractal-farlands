use axum::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        StatusCode,
    },
    response::IntoResponse,
};
use ff_core::RenderRequest;

/// Render the fractal with the provided params.
pub async fn render(
    server: &ff_render::RenderServer,
    request: RenderRequest,
) -> axum::response::Result<impl IntoResponse> {
    let image = server.render(request).await.map_err(|err| {
        tracing::error!("request error: {:?}", err);
        match err {
            ff_render::Error::InvalidArgument(_) => axum::http::StatusCode::NOT_FOUND,
            ff_render::Error::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    let mut buffer = std::io::Cursor::new(Vec::<u8>::new());
    image
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|err| {
            tracing::error!("image serialization error: {}", err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, "image/png")],
        [(CACHE_CONTROL, "max-age=3600")],
        buffer.into_inner(),
    ))
}
