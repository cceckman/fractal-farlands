use crate::WindowParams;
use axum::{
    http::{header::{CACHE_CONTROL, CONTENT_TYPE}, StatusCode},
    response::IntoResponse,
};
use ff_core::mandelbrot;
use num::BigRational;

/// Render the fractal with the provided params.
pub async fn render(
    fractal: String,
    numeric: String,
    query: WindowParams,
) -> axum::response::Result<impl IntoResponse> {
    match fractal.as_str() {
        "mandelbrot" => mandelbrot_render(&numeric, query),
        _ => Err(axum::http::StatusCode::NOT_FOUND.into()),
    }
}

fn mandelbrot_render(
    numeric: &str,
    query: WindowParams,
) -> axum::response::Result<impl IntoResponse> {
    let x_end = &query.x + &query.window;
    let y_end = &query.y + &query.window;
    let x_range = BigRational::new(query.x.clone(), query.scale.clone())
        ..BigRational::new(x_end, query.scale.clone());
    let y_range = BigRational::new(query.y.clone(), query.scale.clone())
        ..BigRational::new(y_end, query.scale.clone());

    let size = ff_core::Size {
        x: query.res,
        y: query.res,
    };

    let computed = match numeric {
        "f32" => mandelbrot::evaluate::<f32>(&x_range, &y_range, size, query.iters),
        "f64" => mandelbrot::evaluate::<f32>(&x_range, &y_range, size, query.iters),
        _ => return Err(axum::http::StatusCode::NOT_FOUND.into()),
    };
    let data: Vec<Option<usize>> = computed.map_err(|err| {
        log::error!("computation error: for parameters {:?}: {}", &query, err);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let image = ff_core::image::Renderer {}
        .render(size, data)
        .map_err(|err| {
            log::error!("rendering error: for parameters {:?}: {}", &query, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut buffer = std::io::Cursor::new(Vec::<u8>::new());
    image
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|err| {
            log::error!(
                "image serialization error: for parameters {:?}: {}",
                &query,
                err
            );
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, "image/png")],
        [(CACHE_CONTROL, "max-age=3600")],
        buffer.into_inner(),
    ))
}
