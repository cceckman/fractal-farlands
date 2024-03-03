use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ff_core::{CommonParams, RenderRequest, Size};
use ff_render::RenderServer;
use num::BigRational;

criterion_main!(benches);
criterion_group!(benches, bench_multithread);

/// Benchmark several implementations in the base window.
pub fn bench_multithread(c: &mut Criterion) {
    let mut group = c.benchmark_group("multithreading@base");

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
    group.throughput(criterion::Throughput::Elements(
        req.common.size.width as u64 * req.common.size.height as u64,
    ));
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    for threads in 1..=(2 * num_cpus::get()) {
        let exec = RenderServer::with_threads(threads).unwrap();

        for numeric in ["f32", "f64", "MaskedFloat<4,50>"] {
            req.common.numeric = numeric.to_string();

            group.bench_with_input(BenchmarkId::new(numeric, threads), &req, |b, input| {
                b.to_async(&rt)
                    .iter_with_large_drop(|| exec.render(black_box(input.clone())))
            });
        }
    }

    group.finish();
}
