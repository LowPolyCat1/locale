#![allow(deprecated)]
#[allow(unused)]
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use locale_rs::Locale;
#[cfg(feature = "currency")]
use locale_rs::currency_formats::ToCurrencyString;
#[cfg(feature = "nums")]
use locale_rs::num_formats::ToFormattedString;
#[cfg(feature = "datetime")]
use locale_rs::{datetime_formats::DateTime, *};
use std::collections::HashSet;
use std::str::FromStr;

// Common test locales used across all benchmarks for consistency
const COMMON_LOCALES: &[Locale] = &[Locale::en, Locale::de, Locale::fr, Locale::ja, Locale::ar];

#[cfg(feature = "datetime")]
fn bench_datetime_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("DateTime Formatting");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    let locale = Locale::en;
    let dt = DateTime {
        year: 2023,
        month: 12,
        day: 25,
        hour: 14,
        minute: 30,
        second: 05,
    };

    group.bench_function("format_date", |b| {
        b.iter(|| locale.format_date(black_box(&dt)))
    });

    group.bench_function("format_time", |b| {
        b.iter(|| locale.format_time(black_box(&dt)))
    });

    group.bench_function("parse_complex_pattern", |b| {
        b.iter(|| locale.format_date(black_box(&dt)))
    });

    // Cross-locale formatting with consistent locale set
    for &locale in COMMON_LOCALES {
        group.bench_function(format!("multilingual_date_{}", locale.as_str()), |b| {
            b.iter(|| locale.format_date(black_box(&dt)))
        });
    }

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_numeric_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Numeric Formatting");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    let locale = Locale::en;

    // Test typical integer sizes
    group.bench_function("format_small_int", |b| {
        b.iter(|| black_box(42i32).to_formatted_string(&locale))
    });

    group.bench_function("format_medium_int", |b| {
        b.iter(|| black_box(1234i32).to_formatted_string(&locale))
    });

    group.bench_function("format_large_int", |b| {
        b.iter(|| black_box(1234567i32).to_formatted_string(&locale))
    });

    group.bench_function("format_very_large_int", |b| {
        b.iter(|| black_box(1234567890i64).to_formatted_string(&locale))
    });

    // Special cases
    group.bench_function("format_zero", |b| {
        b.iter(|| black_box(0i32).to_formatted_string(&locale))
    });

    group.bench_function("format_negative", |b| {
        b.iter(|| black_box(-1234567i64).to_formatted_string(&locale))
    });

    group.bench_function("format_float", |b| {
        b.iter(|| black_box(1234567.891f64).to_formatted_string(&locale))
    });

    // Cross-locale numeric formatting
    for &locale in COMMON_LOCALES {
        group.bench_function(format!("multilingual_format_{}", locale.as_str()), |b| {
            b.iter(|| black_box(1234567).to_formatted_string(&locale))
        });
    }

    // Native digit test (if locale supports it)
    #[cfg(feature = "nums")]
    {
        let ar_locale = Locale::ar_EG;
        group.bench_function("translate_digits_arabic", |b| {
            b.iter(|| black_box(12345).to_formatted_string(&ar_locale))
        });
    }

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Conversions");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // FromStr conversions with consistent complexity levels
    group.bench_function("from_str_simple", |b| {
        b.iter(|| Locale::try_from(black_box("en")))
    });

    group.bench_function("from_str_regional", |b| {
        b.iter(|| Locale::try_from(black_box("en-GB")))
    });

    group.bench_function("from_str_complex", |b| {
        b.iter(|| Locale::try_from(black_box("zh-Hant-HK")))
    });

    // Conversion methods with consistent patterns
    group.bench_function("as_str", |b| {
        let loc = Locale::en;
        b.iter(|| black_box(loc).as_str())
    });

    group.bench_function("into_static_str", |b| {
        let loc = Locale::en;
        b.iter(|| {
            let s: &'static str = black_box(loc).into();
            black_box(s)
        })
    });

    group.bench_function("into_owned_string", |b| {
        let loc = Locale::en;
        b.iter(|| {
            let s: String = black_box(loc).into();
            black_box(s)
        })
    });

    group.bench_function("display_trait", |b| {
        let loc = Locale::en;
        b.iter(|| format!("{}", black_box(loc)))
    });

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_locale_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Operations");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Fallback chain traversal
    group.bench_function("fallback_regional", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).fallback())
    });

    group.bench_function("fallback_base", |b| {
        let loc = Locale::en;
        b.iter(|| black_box(loc).fallback())
    });

    // Property extraction
    group.bench_function("language_code", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).language_code())
    });

    group.bench_function("region_code", |b| {
        let loc = Locale::en_GB;
        b.iter(|| black_box(loc).region_code())
    });

    // Structural operations
    group.bench_function("hash", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let loc = Locale::en_GB;
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            black_box(loc).hash(&mut hasher);
            black_box(hasher.finish())
        })
    });

    group.bench_function("equality_same", |b| {
        let loc1 = Locale::en_GB;
        let loc2 = Locale::en_GB;
        b.iter(|| black_box(loc1) == black_box(loc2))
    });

    group.bench_function("equality_different", |b| {
        let loc1 = Locale::en_GB;
        let loc2 = Locale::de;
        b.iter(|| black_box(loc1) == black_box(loc2))
    });

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Operations");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Stress test: parse all available locales
    group.bench_function("parse_all_locales", |b| {
        b.iter(|| {
            AVAILABLE_LOCALES
                .iter()
                .filter(|&&s| Locale::from_str(s).is_ok())
                .count()
        })
    });

    // Real-world scenario: building a locale collection
    group.bench_function("build_hashset_50_locales", |b| {
        b.iter(|| {
            let mut set = HashSet::new();
            for &locale_str in &AVAILABLE_LOCALES[..50] {
                if let Ok(loc) = Locale::from_str(locale_str) {
                    set.insert(black_box(loc));
                }
            }
            set
        })
    });

    // Real-world: formatting a value in multiple locales
    group.bench_function("format_across_5_locales", |b| {
        b.iter(|| {
            for &locale in COMMON_LOCALES {
                let _ = black_box(1234567).to_formatted_string(&locale);
            }
        })
    });

    // Real-world: format across many locales (stress test)
    group.bench_function("format_across_10_locales", |b| {
        let many_locales = [
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
            for &locale in &many_locales {
                let _ = black_box(1234567).to_formatted_string(&locale);
            }
        })
    });

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Formatting");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    group.bench_function("currency_en_small", |b| {
        b.iter(|| black_box(10u32).to_currency(&Locale::en))
    });

    group.bench_function("currency_en_medium", |b| {
        b.iter(|| black_box(100u32).to_currency(&Locale::en))
    });

    group.bench_function("currency_en_large", |b| {
        b.iter(|| black_box(1000u32).to_currency(&Locale::en))
    });

    group.bench_function("currency_en_xlarge", |b| {
        b.iter(|| black_box(10000u32).to_currency(&Locale::en))
    });

    // Sample multilingual
    group.bench_function("currency_de_medium", |b| {
        b.iter(|| black_box(100u32).to_currency(&Locale::de))
    });

    group.bench_function("currency_ja_medium", |b| {
        b.iter(|| black_box(100u32).to_currency(&Locale::ja))
    });

    group.bench_function("currency_ar_medium", |b| {
        b.iter(|| black_box(100u32).to_currency(&Locale::ar))
    });

    group.finish();
}

#[cfg(all(feature = "datetime", feature = "currency"))]
criterion_group!(
    benches,
    bench_datetime_formatting,
    bench_numeric_formatting,
    bench_conversions,
    bench_locale_ops,
    bench_batch_operations,
    bench_currency_formatting,
);

#[cfg(all(feature = "datetime", not(feature = "currency")))]
criterion_group!(
    benches,
    bench_datetime_formatting,
    bench_numeric_formatting,
    bench_conversions,
    bench_locale_ops,
    bench_batch_operations,
);

#[cfg(feature = "datetime")]
criterion_main!(benches);

#[cfg(not(feature = "datetime"))]
fn main() {
    println!("Info: Feature datetime is not active. Run with --features datetime");
}
