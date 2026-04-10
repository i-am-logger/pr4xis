use criterion::{Criterion, black_box, criterion_group, criterion_main};

use praxis_domains::science::linguistics::english::English;

fn bench_chat(c: &mut Criterion) {
    let en = English::sample();

    let mut group = c.benchmark_group("chat");

    group.bench_function("process_question", |b| {
        b.iter(|| praxis_chat::process(black_box(&en), black_box("is a dog a mammal?")))
    });
    group.bench_function("process_statement", |b| {
        b.iter(|| praxis_chat::process(black_box(&en), black_box("the dog runs")))
    });
    group.bench_function("process_unknown", |b| {
        b.iter(|| praxis_chat::process(black_box(&en), black_box("xyzzy")))
    });
    group.bench_function("self_describe", |b| {
        b.iter(|| praxis_chat::self_describe(black_box(&en)))
    });

    group.finish();
}

criterion_group!(benches, bench_chat);
criterion_main!(benches);
