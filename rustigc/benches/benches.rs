// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rustigc::{Analysis, Log};
use std::time::Duration;

fn load_test_files(files: &[(&'static str, &str)]) -> Vec<(&'static str, Vec<u8>)> {
    files
        .iter()
        .map(|(name, path)| {
            let content =
                std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
            (*name, content)
        })
        .collect()
}

fn parse_logs(files: &[(&'static str, Vec<u8>)]) -> Vec<(&'static str, Log)> {
    files
        .iter()
        .map(|(name, content)| (*name, Log::new(content).unwrap()))
        .collect()
}

const PARSING_FILES: &[(&str, &str)] = &[
    ("plouf", "../test_data/real/free-06.igc"),
    ("local", "../test_data/real/triangle-02.igc"),
    ("complex", "../test_data/real/fai-01.igc"),
    ("long-fai", "../test_data/real/fai-02.igc"),
];

fn bench_log_parsing(c: &mut Criterion) {
    let files = load_test_files(PARSING_FILES);

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
    let files = load_test_files(PARSING_FILES);
    let logs = parse_logs(&files);

    let mut group = c.benchmark_group("analysis");
    for (name, log) in &logs {
        group.bench_with_input(BenchmarkId::from_parameter(name), log, |b, log| {
            b.iter(|| Analysis::new(&black_box(log).track));
        });
    }

    group.finish();
}

const SCORING_FILES: &[(&str, &str)] = &[
    ("plouf", "../test_data/real/free-06.igc"),
    ("local", "../test_data/real/triangle-02.igc"),
    ("long-3pt", "../test_data/real/free-05.igc"),
    ("long-fai", "../test_data/real/fai-02.igc"),
    ("closing", "../test_data/real/fai-01.igc"),
];

fn bench_score(c: &mut Criterion) {
    let files = load_test_files(SCORING_FILES);
    let logs = parse_logs(&files);

    let mut group = c.benchmark_group("score");
    for (name, log) in &logs {
        // Resolved outside the loop: the bench measures scoring, not flight detection
        let Some((start, stop)) = Analysis::new(&log.track).flight() else {
            continue;
        };

        group.bench_with_input(BenchmarkId::from_parameter(name), log, |b, log| {
            b.iter(|| black_box(log).score("xcontest", start, stop));
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)  // 10 iteration minimum
        .measurement_time(Duration::from_secs(1))  // Minimum target wall clock
        .warm_up_time(Duration::from_millis(500));
    targets =
        bench_log_parsing,
        bench_analysis,
        bench_score,
}

criterion_main!(benches);
