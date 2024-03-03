use axum::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        StatusCode,
    },
    response::IntoResponse,
};
use ff_core::{
    masked_float, RenderRequest,
};
use ff_core::mandelbrot;

/// Render the fractal with the provided params.
pub fn render(request: RenderRequest) -> axum::response::Result<impl IntoResponse> {
    match request.fractal {
        ff_core::FractalParams::Mandelbrot { iters } => mandelbrot_render(request.common, iters),
        _ => Err(axum::http::StatusCode::NOT_FOUND.into()),
    }
}

fn mandelbrot_render(
    request: ff_core::CommonParams,
    iters: usize,
) -> axum::response::Result<impl IntoResponse> {
    tracing::info!("starting mandelbrot with format {}", request.numeric);

    let span = tracing::info_span!("render-mandelbrot");
    let _guard = span.enter();
    let computed = match request.numeric.as_str() {
        "f32" => mandelbrot::evaluate::<f32>(&request, iters),
        "f64" => mandelbrot::evaluate::<f64>(&request, iters),
        "MaskedFloat<3,50>" => {
            mandelbrot::evaluate::<masked_float::MaskedFloat<3, 50>>(&request, iters)
        }
        "MaskedFloat<4,50>" => {
            mandelbrot::evaluate::<masked_float::MaskedFloat<4, 50>>(&request, iters)
        }
        "I11F5" => mandelbrot::evaluate::<fixed::types::I11F5>(&request, iters),
        "I13F3" => mandelbrot::evaluate::<fixed::types::I13F3>(&request, iters),
        "I15F1" => mandelbrot::evaluate::<fixed::types::I15F1>(&request, iters),
        _ => return Err(axum::http::StatusCode::NOT_FOUND.into()),
    };
    tracing::debug!("mandelbrot-computed");
    let data: Vec<Option<usize>> = computed.map_err(|err| {
        tracing::error!("computation error: for parameters {:?}: {}", &request, err);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let image = ff_core::image::Renderer {}
        .render(request.size, data)
        .map_err(|err| {
            tracing::error!("rendering error: for parameters {:?}: {}", &request, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;
    tracing::debug!("mandelbrot-rendered");

    let mut buffer = std::io::Cursor::new(Vec::<u8>::new());
    image
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|err| {
            tracing::error!(
                "image serialization error: for parameters {:?}: {}",
                &request,
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
