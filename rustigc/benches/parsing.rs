use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rustigc::{FRawData, Log};

const TEST_FILES: &[(&str, &str)] = &[
    ("plouf", "../test_data/real/plouf-01.igc"),
    ("local", "../test_data/real/local-01.igc"),
    ("complex", "../test_data/real/complex_example_lxn.igc"),
    ("record", "../test_data/real/fai-record-lod-lad.igc"),
];

fn load_test_files() -> Vec<(&'static str, Vec<u8>)> {
    TEST_FILES
        .iter()
        .map(|(name, path)| {
            let content =
                std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
            (*name, content)
        })
        .collect()
}

fn bench_log_parsing(c: &mut Criterion) {
    let files = load_test_files();

    let mut group = c.benchmark_group("log_parsing");
    for (name, content) in &files {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            content,
            |b, content| {
                b.iter(|| Log::new(content).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_analysis(c: &mut Criterion) {
    let files = load_test_files();
    let logs: Vec<(&str, Log)> = files
        .iter()
        .map(|(name, content)| (*name, Log::new(content).unwrap()))
        .collect();

    let mut group = c.benchmark_group("analysis");
    for (name, log) in &logs {
        group.bench_with_input(BenchmarkId::from_parameter(name), log, |b, log| {
            b.iter(|| {
                let raw_data = FRawData::new(log);
                raw_data.phases()
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_log_parsing, bench_analysis);
criterion_main!(benches);
