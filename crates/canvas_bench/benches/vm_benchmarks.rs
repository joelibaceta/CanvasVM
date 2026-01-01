use canvas_vm::{BytecodeVm, Grid};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn load_sample(name: &str) -> (Vec<u8>, usize, usize) {
    let path = format!("../../tools/fixtures/samples/{}.png", name);
    let img = image::open(&path).unwrap().to_rgba8();
    let (width, height) = img.dimensions();
    (img.into_raw(), width as usize, height as usize)
}

fn bench_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation");

    // Simple programs
    for sample in ["HelloWorld", "HelloWorld2", "HelloWorld3", "Piet"] {
        let (rgba, width, height) = load_sample(sample);

        group.bench_with_input(BenchmarkId::new("compile", sample), &sample, |b, _| {
            b.iter(|| {
                let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, None).unwrap();
                let _vm = BytecodeVm::from_grid(grid).unwrap();
            });
        });
    }

    // Complex programs
    for sample in ["PI", "AZ", "Sum"] {
        let (rgba, width, height) = load_sample(sample);

        group.bench_with_input(BenchmarkId::new("compile", sample), &sample, |b, _| {
            b.iter(|| {
                let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, None).unwrap();
                let _vm = BytecodeVm::from_grid(grid).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution");

    // Fast programs
    for sample in ["HelloWorld", "HelloWorld2", "Piet"] {
        let (rgba, width, height) = load_sample(sample);
        let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, None).unwrap();

        group.bench_with_input(BenchmarkId::new("execute", sample), &sample, |b, _| {
            b.iter(|| {
                let mut vm = BytecodeVm::from_grid(grid.clone()).unwrap();
                vm.play(1_000_000).unwrap();
                black_box(vm.ink_string());
            });
        });
    }

    // Compute-intensive programs
    for sample in ["PI", "AZ"] {
        let (rgba, width, height) = load_sample(sample);
        let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, None).unwrap();

        group.bench_with_input(BenchmarkId::new("execute", sample), &sample, |b, _| {
            b.iter(|| {
                let mut vm = BytecodeVm::from_grid(grid.clone()).unwrap();
                vm.play(1_000_000).unwrap();
                black_box(vm.ink_string());
            });
        });
    }

    group.finish();
}

fn bench_codel_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("codel_detection");

    // Small images (fast detection)
    for sample in ["HelloWorld", "HelloWorld2", "Piet", "Sum"] {
        let (rgba, width, height) = load_sample(sample);

        group.bench_with_input(BenchmarkId::new("detect", sample), &sample, |b, _| {
            b.iter(|| {
                Grid::from_rgba_with_codel_size(width, height, &rgba, None).unwrap();
            });
        });
    }

    // Large images (slower detection)
    for sample in ["PI", "AZ", "HelloWorld3"] {
        let (rgba, width, height) = load_sample(sample);

        group.bench_with_input(BenchmarkId::new("detect", sample), &sample, |b, _| {
            b.iter(|| {
                Grid::from_rgba_with_codel_size(width, height, &rgba, None).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_compilation,
    bench_execution,
    bench_codel_detection
);
criterion_main!(benches);
