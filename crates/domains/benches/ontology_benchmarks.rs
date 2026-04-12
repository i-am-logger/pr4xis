use criterion::{Criterion, black_box, criterion_group, criterion_main};

use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

// Benchmark all ontologies — category law verification, morphism enumeration,
// entity variant enumeration, and the linguistics pipeline.
//
// Organized by the Benchmark ontology (Georges et al. 2007):
// - Each benchmark group = one Invocation
// - Each measurement = one Iteration
// - criterion handles Warmup → SteadyState → Measurement automatically
// - Results include ConfidenceInterval (criterion default: 95%)

// ============================================================================
// Category law verification — the foundational check for every ontology
// ============================================================================

fn bench_category_laws(c: &mut Criterion) {
    let mut group = c.benchmark_group("category_laws");

    // Core praxis ontologies
    group.bench_function("systems", |b| {
        b.iter(|| {
            check_category_laws::<pr4xis_domains::science::systems::ontology::SystemsCategory>()
        })
    });

    // New ontologies from this session
    group.bench_function("schema", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::schema::ontology::SchemaCategory,
            >()
        })
    });
    group.bench_function("instance", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::schema::instance::InstanceCategory,
            >()
        })
    });
    group.bench_function("repository", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::storage::ontology::RepositoryCategory,
            >()
        })
    });
    group.bench_function("consistency", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::storage::consistency::ConsistencyCategory,
            >()
        })
    });
    group.bench_function("durability", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::storage::durability::DurabilityCategory,
            >()
        })
    });
    group.bench_function("volatility", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::storage::volatility::VolatilityCategory,
            >()
        })
    });
    group.bench_function("measurement", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::measurement::ontology::MeasurementCategory,
            >()
        })
    });
    group.bench_function("benchmark", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::information::measurement::benchmark::BenchmarkCategory,
            >()
        })
    });

    // Math ontologies (from sensor-fusion)
    group.bench_function("geometry", |b| {
        b.iter(|| check_category_laws::<pr4xis_domains::science::math::geometry::ontology::GeometryCategory>())
    });
    group.bench_function("linear_algebra", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::math::linear_algebra::ontology::LinearAlgebraCategory,
            >()
        })
    });
    group.bench_function("probability", |b| {
        b.iter(|| {
            check_category_laws::<
                pr4xis_domains::science::math::probability::ontology::ProbabilityCategory,
            >()
        })
    });
    group.bench_function("rotation", |b| {
        b.iter(|| check_category_laws::<pr4xis_domains::science::math::rotation::ontology::RotationCategory>())
    });

    // Technology ontologies
    group.bench_function("chess", |b| {
        b.iter(|| check_category_laws::<pr4xis_domains::technology::games::chess::ontology::ChessCategory>())
    });

    group.finish();
}

// ============================================================================
// Morphism enumeration — how fast can we list all relationships
// ============================================================================

fn bench_morphisms(c: &mut Criterion) {
    let mut group = c.benchmark_group("morphisms");

    group.bench_function("systems", |b| {
        b.iter(|| {
            black_box(pr4xis_domains::science::systems::ontology::SystemsCategory::morphisms())
        })
    });
    group.bench_function("schema", |b| {
        b.iter(|| {
            black_box(
                pr4xis_domains::science::information::schema::ontology::SchemaCategory::morphisms(),
            )
        })
    });
    group.bench_function("consistency", |b| {
        b.iter(|| black_box(pr4xis_domains::science::information::storage::consistency::ConsistencyCategory::morphisms()))
    });
    group.bench_function("geometry", |b| {
        b.iter(|| {
            black_box(
                pr4xis_domains::science::math::geometry::ontology::GeometryCategory::morphisms(),
            )
        })
    });
    group.bench_function("chess", |b| {
        b.iter(|| {
            black_box(
                pr4xis_domains::technology::games::chess::ontology::ChessCategory::morphisms(),
            )
        })
    });
    group.bench_function("measurement", |b| {
        b.iter(|| black_box(pr4xis_domains::science::information::measurement::ontology::MeasurementCategory::morphisms()))
    });

    group.finish();
}

// ============================================================================
// Entity variant enumeration
// ============================================================================

fn bench_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("variants");

    group.bench_function("system_concepts", |b| {
        b.iter(|| black_box(pr4xis_domains::science::systems::ontology::SystemConcept::variants()))
    });
    group.bench_function("schema_concepts", |b| {
        b.iter(|| {
            black_box(
                pr4xis_domains::science::information::schema::ontology::SchemaConcept::variants(),
            )
        })
    });
    group.bench_function("consistency_models", |b| {
        b.iter(|| black_box(pr4xis_domains::science::information::storage::consistency::ConsistencyModel::variants()))
    });
    group.bench_function("storage_media", |b| {
        b.iter(|| {
            black_box(
                pr4xis_domains::science::information::storage::volatility::StorageMedia::variants(),
            )
        })
    });
    group.bench_function("geometric_primitives", |b| {
        b.iter(|| {
            black_box(
                pr4xis_domains::science::math::geometry::ontology::GeometricPrimitive::variants(),
            )
        })
    });

    group.finish();
}

// ============================================================================
// Linguistics pipeline — tokenize, parse, interpret
// ============================================================================

fn bench_linguistics(c: &mut Criterion) {
    use pr4xis_domains::science::linguistics::english::English;
    use pr4xis_domains::science::linguistics::lambek::{reduce, tokenize};
    use pr4xis_domains::science::linguistics::language::Language;

    let en = English::sample();

    let mut group = c.benchmark_group("linguistics");

    // Tokenization
    group.bench_function("tokenize_simple", |b| {
        b.iter(|| tokenize::tokenize(black_box("the dog runs"), &en))
    });
    group.bench_function("tokenize_with_alternatives", |b| {
        b.iter(|| tokenize::tokenize_with_alternatives(black_box("the dog runs"), &en))
    });

    // Chart parsing (CYK)
    let (tokens, type_sets) = tokenize::tokenize_with_alternatives("the dog runs", &en);
    let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
    group.bench_function("chart_reduce_3_words", |b| {
        b.iter(|| reduce::chart_reduce(black_box(&words), black_box(&type_sets)))
    });

    let (tokens5, type_sets5) = tokenize::tokenize_with_alternatives("she sees the big dog", &en);
    let words5: Vec<String> = tokens5.iter().map(|t| t.word.clone()).collect();
    group.bench_function("chart_reduce_5_words", |b| {
        b.iter(|| reduce::chart_reduce(black_box(&words5), black_box(&type_sets5)))
    });

    // Lexical lookup
    group.bench_function("lexical_lookup_function_word", |b| {
        b.iter(|| en.lexical_lookup(black_box("the")))
    });
    group.bench_function("lexical_lookup_content_word", |b| {
        b.iter(|| en.lexical_lookup(black_box("dog")))
    });
    group.bench_function("lexical_lookup_all", |b| {
        b.iter(|| en.lexical_lookup_all(black_box("run")))
    });

    // English construction
    group.bench_function("english_sample", |b| b.iter(|| English::sample()));

    group.finish();
}

criterion_group!(
    benches,
    bench_category_laws,
    bench_morphisms,
    bench_variants,
    bench_linguistics,
);
criterion_main!(benches);
