use criterion::{criterion_group, criterion_main, Criterion};
use sm_motion_photo::SmMotion;
use std::env;
use std::fs::File;

fn get_photo_file() -> File {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    File::open(format!("{}/{}", dir, "tests/data/photo.jpg")).unwrap()
}

fn get_sm_motion() -> SmMotion {
    match SmMotion::with(&get_photo_file()) {
        Some(sm) => sm,
        None => panic!("Not created motion"),
    }
}

pub fn video_context_benchmark(c: &mut Criterion) {
    c.bench_function("video_context", |b| {
        b.iter(|| get_sm_motion().find_video_context())
    });
}

criterion_group!(benches, video_context_benchmark);
criterion_main!(benches);
