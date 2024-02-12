use clap::Parser;
use ff_core::image::Renderer;
use ff_core::mandelbrot::{Mandelbrot, MandelbrotEval};
use ff_core::Size;
use num::BigRational;
use std::io::stderr;
use std::io::Write;
use std::{
    path::{Path, PathBuf},
    sync::Barrier,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    x_start: num::BigRational,
    #[arg(long)]
    x_end: num::BigRational,
    #[arg(long)]
    y_start: num::BigRational,
    #[arg(long)]
    y_end: num::BigRational,

    #[arg(long)]
    width: usize,
    #[arg(long)]
    height: usize,

    #[arg(long)]
    out_dir: PathBuf,

    #[arg(long, use_value_delimiter = true, value_delimiter = ',')]
    iterations: Vec<usize>,
}

struct WorkerArgs<'a> {
    evaluator: Box<dyn Mandelbrot + Send>,

    // Concurrency control:
    // Consume one iteration, then wait on the barrier before continuing.
    // This might be slower, but it's "nice" to step in time.
    iterations: &'a [usize],
    barrier: &'a Barrier,

    out_dir: &'a Path,
}

impl WorkerArgs<'_> {
    pub fn run(mut self) {
        for iter in self.iterations {
            self.advance_eval_to(*iter);
            let bmp = self.render();
            self.write_image(bmp).unwrap();
            self.barrier.wait();
        }
    }

    fn advance_eval_to(&mut self, iters: usize) {
        let count = self.evaluator.num_iters();
        let remaining = iters.saturating_sub(count);
        self.evaluator.advance(remaining);
    }

    fn render(&self) -> image::DynamicImage {
        let r = Renderer::default();
        r.render(self.evaluator.size(), self.evaluator.state())
            .expect("could not render image")
    }

    fn write_image(&self, img: image::DynamicImage) -> Result<(), String> {
        // TODO: Convert to a lossless format; Firefox doesn't want to display TIFFs.
        let filename = format!(
            "{}_{}.png",
            self.evaluator.name(),
            self.evaluator.num_iters()
        );
        let path = self.out_dir.join(filename);
        img.save(&path)
            .map_err(|err| format!("failed to save image to {}: {}", path.display(), err))?;

        writeln!(stderr().lock(), "wrote {}", path.display())
            .map_err(|err| format!("stderr error: {}", err))
    }
}

fn main() {
    let args = Args::parse();
    let x_bounds = args.x_start..args.x_end;
    let y_bounds = args.y_start..args.y_end;
    let size = Size {
        x: args.width,
        y: args.height,
    };

    // 4 threads: 3 workers, and this one for logging and completion.
    let barrier = Barrier::new(4);

    std::thread::scope(|scope| {
        let evals: Vec<Box<dyn Mandelbrot + Send>> = vec![
            Box::new(MandelbrotEval::<f32>::new(&x_bounds, &y_bounds, size).unwrap()),
            Box::new(MandelbrotEval::<f64>::new(&x_bounds, &y_bounds, size).unwrap()),
            Box::new(MandelbrotEval::<BigRational>::new(&x_bounds, &y_bounds, size).unwrap()),
        ];
        let scopes: Vec<_> = evals
            .into_iter()
            .map(|evaluator| {
                let worker = WorkerArgs {
                    evaluator,
                    iterations: &args.iterations,
                    barrier: &barrier,

                    out_dir: &args.out_dir,
                };
                scope.spawn(move || {
                    let worker = worker;
                    worker.run()
                })
            })
            .collect();

        // Great! We've launched all the worker threads.
        // Step through along with them...
        (|| {
            let mut out = stderr().lock();
            writeln!(out, "Evaluating range:")?;
            writeln!(out, " x: [{}, {}]", x_bounds.start, x_bounds.end)?;
            writeln!(out, " y: [{}, {}]", y_bounds.start, y_bounds.end)?;
            writeln!(out, "size: {} x {}", size.x, size.y)
        })()
        .unwrap();
        for iterations in args.iterations.iter() {
            writeln!(
                stderr().lock(),
                "Running through {} iterations in {} formats",
                iterations,
                scopes.len()
            )
            .unwrap();
            barrier.wait();
        }

        // All threads are done.
        // TODO: Create evaluators, spawn workers, wait for them to be done.
    });

    writeln!(stderr(), "Outputs in {}", args.out_dir.display()).unwrap();
}
