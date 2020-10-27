use criterion::{criterion_group, criterion_main, Criterion};
use sm_motion_photo::SmMotion;
use std::env;
use std::fs::File;

fn get_photo_file() -> File {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    File::open(format!("{}/{}", dir, "tests/data/photo.jpg")).unwrap()
}

pub fn index_benchmark(c: &mut Criterion) {
    c.bench_function("index", |b| b.iter(|| SmMotion::with(&get_photo_file())));
}

criterion_group!(benches, index_benchmark);
criterion_main!(benches);
