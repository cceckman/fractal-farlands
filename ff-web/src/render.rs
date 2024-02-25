use crate::WindowParams;
use axum::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        StatusCode,
    },
    response::IntoResponse,
};
use ff_core::{mandelbrot, masked_float};
use num::BigRational;
use num_bigint::BigInt;

/// Render the fractal with the provided params.
pub fn render(
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
    tracing::info!("starting mandelbrot with format {}", numeric);
    let step = &query.window / 2;

    let range = |center: &BigInt| {
        BigRational::new(center - &step, query.scale.clone())
            ..BigRational::new(center + &step, query.scale.clone())
    };
    let x_range = range(&query.x);
    let y_range = range(&query.y);

    let size = ff_core::Size {
        x: query.res,
        y: query.res,
    };

    let span = tracing::info_span!("render-mandelbrot", numeric);
    let _guard = span.enter();
    let computed = match numeric {
        "f32" => mandelbrot::evaluate::<f32>(&x_range, &y_range, size, query.iters),
        "f64" => mandelbrot::evaluate::<f64>(&x_range, &y_range, size, query.iters),
        "MaskedFloat<3,50>" => mandelbrot::evaluate::<masked_float::MaskedFloat<3,50>>(&x_range, &y_range, size, query.iters),
        "MaskedFloat<4,50>" => mandelbrot::evaluate::<masked_float::MaskedFloat<4,50>>(&x_range, &y_range, size, query.iters),
        _ => return Err(axum::http::StatusCode::NOT_FOUND.into()),
    };
    tracing::debug!("mandelbrot-computed");
    let data: Vec<Option<usize>> = computed.map_err(|err| {
        tracing::error!("computation error: for parameters {:?}: {}", &query, err);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let image = ff_core::image::Renderer {}
        .render(size, data)
        .map_err(|err| {
            tracing::error!("rendering error: for parameters {:?}: {}", &query, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;
    tracing::debug!("mandelbrot-rendered");

    let mut buffer = std::io::Cursor::new(Vec::<u8>::new());
    image
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|err| {
            tracing::error!(
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
