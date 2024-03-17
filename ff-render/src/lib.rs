//! Render server for Fractal Farlands.
//!
//! Depending on the fractal, renders can be concurrent up to the "per-pixel" level.
//! To support this parallelism, we provide the render server, which renders fractals in its own thread-pool.
//!
//! For raster images, rendering occurs in these steps:
//! -   Map the input request into a request-per-row.
//!     This is a guess at a compromise between "render at each pixel" and "parallelize";
//!     it provides a convenient breakpoint, with relatively high locality.

use std::{
    future::Future,
    sync::{mpsc::Receiver, Arc},
};

use ff_core::RenderRequest;
mod oneshot;

pub struct RenderServer {
    queue: std::sync::mpsc::Sender<ImageRequest>,
}

struct ImageRequest {
    request: RenderRequest,
    result: oneshot::Sender<Completion>,
}

/// Errors that can occur during execution.
#[derive(Clone, Debug)]
pub enum Error {
    InvalidArgument(String),
    Internal(String),
}

pub type Completion = Result<image::DynamicImage, Error>;

impl RenderServer {
    pub fn new() -> Result<Self, String> {
        Self::with_threads(rayon::current_num_threads())
    }

    pub fn with_threads(threads: usize) -> Result<Self, String> {
        if threads < 1 {
            return Err("must provide >=1 thread".to_string());
        }
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|v| format!("error creating thread pool: {}", v))?;

        let (queue, recv) = std::sync::mpsc::channel();
        // The dispatch thread is free-running. It shuts down when the input queue closes.
        std::thread::spawn(move || dispatch(pool, recv));

        Ok(RenderServer { queue })
    }

    pub fn render(&self, request: RenderRequest) -> impl Future<Output = Completion> {
        let (result, recv) = oneshot::new();
        let req = ImageRequest { request, result };
        if let Err(std::sync::mpsc::SendError(req)) = self.queue.send(req) {
            req.result.send(Err(Error::Internal(
                "rendering server has terminated".to_string(),
            )));
        }
        async move {
            match recv.await {
                Ok(v) => v,
                Err(e) => Err(Error::Internal(e.to_string())),
            }
        }
    }
}

fn dispatch(pool: rayon::ThreadPool, receiver: Receiver<ImageRequest>) {
    let pool = Arc::new(pool);
    let span = tracing::info_span!("dispatch thread");
    let _ = span.enter();

    for req in receiver.iter() {
        // spawn_fifo so that images complete in ~the same order as requested;
        // we don't want partially-rendered images.
        pool.spawn_fifo({
            || render(req)
        })
    }
}

fn render(req: ImageRequest) {
    let ImageRequest { request, result } = req;
    let res = match request.fractal {
        ff_core::FractalParams::Mandelbrot { iters } => {
            mandelbrot_render(request.common, iters)
        }
        ff_core::FractalParams::Newton { iters } => {
            newton_render(request.common, iters)
        }
        _ => Err(Error::InvalidArgument("unknown fractal".to_owned())),
    };
    result.send(res);
}

fn mandelbrot_render(
    request: ff_core::CommonParams,
    iters: usize,
) -> Result<image::DynamicImage, Error> {
    tracing::info!("starting mandelbrot with format {}", request.numeric);

    let span = tracing::info_span!("render-mandelbrot");
    let _guard = span.enter();
    let size = request.size.clone();

    let output = ff_core::mandelbrot::compute(&request, iters)
        .map_err(|msg| Error::Internal(msg))?;
    tracing::debug!("mandelbrot-computed");

    let image = ff_core::image::Renderer {}
        .render(size, output)
        .map_err(|err| {
            tracing::error!("rendering error: {}", err);
            Error::Internal(format!("rendering error: {}", err))
        })?;
    tracing::debug!("mandelbrot-rendered");

    Ok(image)
}

fn newton_render(
    request: ff_core::CommonParams,
    iters: usize,
) -> Result<image::DynamicImage, Error> {
    tracing::info!("starting newton with format {}", request.numeric);

    let span = tracing::info_span!("render-newton");
    let _guard = span.enter();
    let size = request.size.clone();

    let output = ff_core::newton::compute(&request, iters)
        .map_err(|msg| Error::Internal(msg))?;
    tracing::debug!("newton-computed");

    let image = ff_core::image::NewtonRenderer {}
        .render(size, output)
        .map_err(|err| {
            tracing::error!("rendering error: {}", err);
            Error::Internal(format!("rendering error: {}", err))
        })?;
    tracing::debug!("newton-rendered");

    Ok(image)
}