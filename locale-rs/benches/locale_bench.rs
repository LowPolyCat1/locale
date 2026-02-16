#![allow(deprecated)]
#[allow(unused)]
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use locale_rs::Locale;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

// Test locales for comprehensive coverage
const TEST_LOCALES_SIMPLE: &[&str] = &["en", "de", "fr", "ja", "ar", "es", "it", "ru", "hi", "zh"];
const TEST_LOCALES_REGIONAL: &[&str] = &[
    "en-GB", "en-US", "de-DE", "de-AT", "fr-FR", "fr-CA", "es-ES", "es-MX", "pt-PT", "pt-BR",
    "zh-Hans", "zh-Hant",
];
const TEST_LOCALES_COMPLEX: &[&str] = &["zh-Hans-CN", "zh-Hant-HK", "sr-Latn-RS", "bs-Cyrl-BA"];

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Parsing");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Simple locale parsing
    group.bench_function("parse_simple", |b| {
        b.iter(|| {
            for &locale_str in TEST_LOCALES_SIMPLE {
                let _ = Locale::from_str(black_box(locale_str));
            }
        })
    });

    // Regional variant parsing
    group.bench_function("parse_regional", |b| {
        b.iter(|| {
            for &locale_str in TEST_LOCALES_REGIONAL {
                let _ = Locale::from_str(black_box(locale_str));
            }
        })
    });

    // Complex locale parsing
    group.bench_function("parse_complex", |b| {
        b.iter(|| {
            for &locale_str in TEST_LOCALES_COMPLEX {
                let _ = Locale::from_str(black_box(locale_str));
            }
        })
    });

    // Individual complexity levels
    for (input, label) in &[
        ("en", "1-segment"),
        ("en-GB", "2-segment"),
        ("zh-Hans-CN", "3-segment"),
    ] {
        group.bench_with_input(BenchmarkId::new("parse_segments", label), input, |b, s| {
            b.iter(|| Locale::from_str(black_box(s)))
        });
    }

    group.finish();
}

fn bench_properties(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Properties");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Language code extraction from various locale types
    group.bench_function("language_code_simple", |b| {
        let loc = Locale::en;
        b.iter(|| black_box(loc).language_code())
    });

    group.bench_function("language_code_regional", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).language_code())
    });

    group.bench_function("language_code_complex", |b| {
        let loc = Locale::zh_Hans;
        b.iter(|| black_box(loc).language_code())
    });

    // Region code extraction
    group.bench_function("region_code_simple", |b| {
        let loc = Locale::en;
        b.iter(|| black_box(loc).region_code())
    });

    group.bench_function("region_code_regional", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).region_code())
    });

    group.bench_function("region_code_complex", |b| {
        let loc = Locale::zh_Hans;
        b.iter(|| black_box(loc).region_code())
    });

    // String conversion
    group.bench_function("as_str", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).as_str())
    });

    group.finish();
}

fn bench_fallback(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Fallback");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Single fallback
    group.bench_function("fallback_one_level", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).fallback())
    });

    // Fallback chain traversal
    group.bench_function("fallback_chain", |b| {
        let loc = Locale::zh_Hans;
        b.iter(|| {
            let mut current = Some(black_box(loc));
            while let Some(l) = current {
                current = l.fallback();
            }
        })
    });

    let test_locales = [
        (Locale::en, "base_no_fallback"),
        (Locale::en_GB, "regional_one_level"),
        (Locale::zh_Hans, "complex_fallback"),
    ];
    for (locale, label) in test_locales {
        group.bench_with_input(BenchmarkId::new("fallback", label), &locale, |b, &l| {
            b.iter(|| black_box(l).fallback())
        });
    }

    group.finish();
}

fn bench_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Collections");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Building HashSet from all locales
    group.bench_function("hashset_all_locales", |b| {
        b.iter(|| {
            let mut set = HashSet::new();
            for locale_str in locale_rs::AVAILABLE_LOCALES {
                if let Ok(loc) = Locale::from_str(locale_str) {
                    set.insert(black_box(loc));
                }
            }
            set
        })
    });

    // Building HashMap with locale metadata
    group.bench_function("hashmap_locale_metadata", |b| {
        b.iter(|| {
            let mut map: HashMap<Locale, &'static str> = HashMap::new();
            for &locale_str in locale_rs::AVAILABLE_LOCALES.iter().take(100) {
                if let Ok(loc) = Locale::from_str(locale_str) {
                    map.insert(black_box(loc), locale_str);
                }
            }
            map
        })
    });

    // Lookup operations in HashSet
    group.bench_function("hashset_lookup_present", |b| {
        let mut set = HashSet::new();
        for &locale_str in &["en", "de", "fr", "ja", "ar"] {
            if let Ok(loc) = Locale::from_str(locale_str) {
                set.insert(loc);
            }
        }
        b.iter(|| set.contains(&black_box(Locale::en)))
    });

    group.bench_function("hashset_lookup_absent", |b| {
        let mut set = HashSet::new();
        for &locale_str in &["de", "fr", "ja"] {
            if let Ok(loc) = Locale::from_str(locale_str) {
                set.insert(loc);
            }
        }
        b.iter(|| set.contains(&black_box(Locale::en)))
    });

    group.finish();
}

fn bench_comparisons(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Comparisons");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Equality comparisons
    group.bench_function("eq_same_locale", |b| {
        let loc1 = Locale::en_GB;
        let loc2 = Locale::en_GB;
        b.iter(|| black_box(loc1) == black_box(loc2))
    });

    group.bench_function("eq_different_locale", |b| {
        let loc1 = Locale::en_GB;
        let loc2 = Locale::de;
        b.iter(|| black_box(loc1) == black_box(loc2))
    });

    // Hash operations (used in collections)
    group.bench_function("hash_operation", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let loc = Locale::en_GB;
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            black_box(loc).hash(&mut hasher);
            black_box(hasher.finish())
        })
    });

    // Batch comparisons
    group.bench_function("compare_batch_10", |b| {
        let locales = [
            Locale::en,
            Locale::de,
            Locale::fr,
            Locale::es,
            Locale::it,
            Locale::ja,
            Locale::zh,
            Locale::ar,
            Locale::ru,
            Locale::hi,
        ];
        b.iter(|| {
            let mut count = 0;
            for &l1 in &locales {
                for &l2 in &locales {
                    if black_box(l1) == black_box(l2) {
                        count += 1;
                    }
                }
            }
            count
        })
    });

    group.finish();
}

fn bench_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Conversions");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Into String conversions
    group.bench_function("into_string", |b| {
        let loc = Locale::en_GB;
        b.iter(|| {
            let s: String = black_box(loc).into();
            black_box(s)
        })
    });

    // Into static str conversions
    group.bench_function("into_static_str", |b| {
        let loc = Locale::en_GB;
        b.iter(|| {
            let s: &'static str = black_box(loc).into();
            black_box(s)
        })
    });

    // Display formatting
    group.bench_function("display_format", |b| {
        let loc = Locale::en_GB;
        b.iter(|| format!("{}", black_box(loc)))
    });

    // Debug formatting
    group.bench_function("debug_format", |b| {
        let loc = Locale::en_GB;
        b.iter(|| format!("{:?}", black_box(loc)))
    });

    group.finish();
}

fn bench_large_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Scale Operations");
    group.sample_size(10); // Smaller sample for expensive operations
    group.measurement_time(std::time::Duration::from_secs(5));
    group.warm_up_time(std::time::Duration::from_secs(1));

    // Parse all 766 locales
    group.bench_function("parse_all_766_locales", |b| {
        b.iter(|| {
            let mut success = 0;
            let mut failed = 0;
            for locale_str in locale_rs::AVAILABLE_LOCALES {
                match Locale::from_str(black_box(locale_str)) {
                    Ok(_) => success += 1,
                    Err(_) => failed += 1,
                }
            }
            (success, failed)
        })
    });

    // Create and populate large collections
    group.bench_function("build_large_hashmap_200", |b| {
        b.iter(|| {
            let mut map: HashMap<Locale, usize> = HashMap::new();
            for (idx, &locale_str) in locale_rs::AVAILABLE_LOCALES.iter().take(200).enumerate() {
                if let Ok(loc) = Locale::from_str(locale_str) {
                    map.insert(black_box(loc), idx);
                }
            }
            map
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing,
    bench_properties,
    bench_fallback,
    bench_collections,
    bench_comparisons,
    bench_conversions,
    bench_large_scale,
);

criterion_main!(benches);
