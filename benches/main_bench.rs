use criterion::{ black_box, criterion_group, criterion_main, Criterion };
use std::path::Path;
use tokio::runtime::Runtime;
use extract_metadata::{ process_safetensors_file, walk_directory };

fn bench_process_safetensors_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let path = Path::new("E:\\models\\RetouchPhoto_PonyV6_v3.1.safetensors");

    c.bench_function("process_safetensors_file", |b| {
        b.to_async(&rt).iter(|| async {
            process_safetensors_file(black_box(path)).await.unwrap();
        });
    });
}

fn bench_walk_directory(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let path = Path::new("E:\\models");

    c.bench_function("walk_directory", |b| {
        b.to_async(&rt).iter(|| async {
            walk_directory(black_box(path), "safetensors", |file_path| {
                async move { process_safetensors_file(black_box(&file_path)).await }
            }).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_process_safetensors_file, bench_walk_directory);
criterion_main!(benches);
