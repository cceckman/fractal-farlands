use bmp_rust::bmp::BMP;
use clap::Parser;
use colors_transform::{Color, Hsl};
use ff_core::mandelbrot::Mandelbrot;
use std::{
    path::{Path, PathBuf},
    sync::Barrier,
};

#[derive(Debug, Parser)]
struct Args {
    x_start: num::BigRational,
    x_end: num::BigRational,
    y_start: num::BigRational,
    y_end: num::BigRational,

    width: usize,
    height: usize,

    iters: Vec<usize>,

    out_dir: PathBuf,
}

struct WorkerArgs<'a> {
    evaluator: Box<dyn Mandelbrot>,

    // Concurrency control:
    // Consume one iteration, then wait on the barrier before continuing.
    // This might be slower, but it's "nice" to step in time.
    iters: &'a [usize],
    barrier: &'a Barrier,

    outpath: &'a Path,
}

impl WorkerArgs<'_> {
    pub fn run(mut self) {
        for iter in self.iters {
            self.advance_eval_to(*iter);
            let bmp = self.render();
            self.write_image(bmp);
            self.barrier.wait();
        }
    }

    fn advance_eval_to(&mut self, iters: usize) {
        let count = self.evaluator.num_iters();
        let remaining = count.saturating_sub(iters);
        self.evaluator.advance(remaining);
    }

    fn render(&self) -> BMP {
        let inputs = self.evaluator.state();
        let (min, max) =
            inputs
                .iter()
                .fold((self.evaluator.num_iters(), 0), |(min, max), v| match v {
                    None => (min, max),
                    Some(iters) => (std::cmp::min(min, *iters), std::cmp::max(max, *iters)),
                });
        let (min, max): (f32, f32) = (min as f32, max as f32);
        let range = max - min;
        // TODO: Don't need to inclue a new dep for this, BMP has it
        // Map to colors:
        let pixels: Vec<_> = inputs
            .into_iter()
            .map(|v| match v {
                None => Hsl::from(0.0, 0.0, 0.0),
                Some(v) => {
                    let iters = v as f32;
                    let degrees = ((iters - min) * 360.0) / range;

                    Hsl::from(degrees, 100.0, 50.0)
                }
            })
            .map(|hsl| {
                let (r, g, b) = hsl.to_rgb().as_tuple();
                (r as u8, g as u8, b as u8)
            })
            .collect();

        let (xsize, ysize) = self.evaluator.size();
        let xcoords = (0..xsize).cycle().take(xsize * ysize);
        let ycoords = (0..ysize).map(|v| std::iter::repeat(v).take(xsize)).flatten();
        let coords = ycoords.zip(xcoords);
        let coords_and_values = coords.zip(pixels);

        let mut image = BMP::new(ysize as i32, xsize as u32, None);

        for cv in coords_and_values {
            let ((x, y), (r, g, b)) = cv;
            // TODO: Use "efficient" variants
            image.change_color_of_pixel(x as u16, y as u16, [r, g, b, 0]).expect("failed to write pixel")
        }

        image
    }

    fn write_image(&self, bmp: BMP) {
        let filename = format!(
            "{}_{}.bmp",
            self.evaluator.name(),
            self.evaluator.num_iters()
        );
        let path = self.outpath.join(filename);
        bmp.save_to_new(&path.to_string_lossy()).expect("failed to write output file")
    }
}

fn main() {
    let args = Args::parse();

    std::thread::scope(|scope| {
        let x_bounds = args.x_start..args.x_end;
        let y_bounds = args.y_start..args.y_end;

        // 4 threads: 3 workers, and this one for logging and completion.
        let b = Barrier::new(4);

        // TODO: Create evaluators, spawn workers, wait for them to be done.
    });
}
