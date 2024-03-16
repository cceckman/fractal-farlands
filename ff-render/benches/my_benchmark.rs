use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ff_core::{CommonParams, RenderRequest, Size};
use ff_render::RenderServer;
use num::BigRational;

criterion_main!(benches);
criterion_group!(benches, bench_multithread);

/// Benchmark several in the base window, across threads.
pub fn bench_multithread(c: &mut Criterion) {
    let mut group = c.benchmark_group("multithreading-base");

    let range = BigRational::new((-2).into(), 1.into())..BigRational::new((2).into(), 1.into());
    let mut req = RenderRequest {
        common: CommonParams {
            size: Size {
                width: 512,
                height: 512,
            },
            x: range.clone(),
            y: range.clone(),
            numeric: "".to_string(),
        },
        fractal: ff_core::FractalParams::Mandelbrot { iters: 16 },
    };
    // Count pixels:
    group.throughput(criterion::Throughput::Elements(
        req.common.size.width as u64 * req.common.size.height as u64,
    ));
    // Don't spend too long preparing:
    group.warm_up_time(Duration::from_secs(1));

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    // Count up powers of two:
    let thread_range = (0..).map(|x| 1 << x).take_while({
        let x = num_cpus::get().next_power_of_two();
        move |y| (*y <= x)
    });
    for threads in thread_range {
        let exec = RenderServer::with_threads(threads).unwrap();

        for numeric in ff_core::mandelbrot::formats() {
            req.common.numeric = numeric.to_string();

            group.bench_with_input(BenchmarkId::new(numeric, threads), &req, |b, input| {
                b.to_async(&rt)
                    .iter_with_large_drop(|| exec.render(black_box(input.clone())))
            });
        }
    }

    group.finish();
}
