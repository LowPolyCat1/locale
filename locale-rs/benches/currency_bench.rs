#![allow(deprecated)]
#[cfg(feature = "currency")]
#[allow(unused)]
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
#[cfg(feature = "currency")]
use locale_rs::{Locale, currency_formats::ToCurrencyString};
#[cfg(feature = "currency")]
use std::str::FromStr;

#[cfg(feature = "currency")]
fn bench_currency_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Basic Formatting");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Small amounts
    group.bench_function("currency_small_amount", |b| {
        b.iter(|| black_box(10u32).to_currency(&Locale::en))
    });

    // Medium amounts
    group.bench_function("currency_medium_amount", |b| {
        b.iter(|| black_box(100u32).to_currency(&Locale::en))
    });

    // Large amounts
    group.bench_function("currency_large_amount", |b| {
        b.iter(|| black_box(1000u32).to_currency(&Locale::en))
    });

    // Very large amounts
    group.bench_function("currency_xlarge_amount", |b| {
        b.iter(|| black_box(10000u32).to_currency(&Locale::en))
    });

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_multilingual(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Multilingual");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    let locales = [
        Locale::en, // English ($ USD)
        Locale::de, // German (€ EUR)
        Locale::fr, // French (€ EUR)
        Locale::ja, // Japanese (¥ JPY)
        Locale::ar, // Arabic
        Locale::pt, // Portuguese
        Locale::ru, // Russian
        Locale::it, // Italian
        Locale::es, // Spanish
        Locale::ko, // Korean
    ];

    // Same amount across different locales
    group.bench_function("currency_100_all_locales", |b| {
        b.iter(|| {
            for &loc in &locales {
                let _ = black_box(100u32).to_currency(&loc);
            }
        })
    });

    // Individual locale benchmarks
    for &locale in &locales {
        group.bench_function(format!("currency_en_{}", locale.as_str()), |b| {
            b.iter(|| black_box(100u32).to_currency(&locale))
        });
    }

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_regional_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Regional Variants");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Different regions using same currency
    let en_variants = [
        Locale::en,    // Base English
        Locale::en_GB, // British English
        Locale::en_AG, // Antigua English variant
        Locale::en_AI, // Anguilla variant
        Locale::en_BE, // Belgium English
        Locale::en_BW, // Botswana English
    ];

    group.bench_function("currency_en_variants_100", |b| {
        b.iter(|| {
            for &loc in &en_variants {
                let _ = black_box(100u32).to_currency(&loc);
            }
        })
    });

    for &locale in &en_variants {
        group.bench_function(format!("currency_{}", locale.as_str()), |b| {
            b.iter(|| black_box(100u32).to_currency(&locale))
        });
    }

    // Different regions with different currencies
    let variants_mixed = [
        Locale::es,    // Spanish (base)
        Locale::de,    // German (no region)
        Locale::de_AT, // Austria variant
        Locale::de_CH, // Switzerland German variant
    ];

    group.bench_function("currency_es_variants_100", |b| {
        b.iter(|| {
            for &loc in &variants_mixed {
                let _ = black_box(100u32).to_currency(&loc);
            }
        })
    });

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_amounts(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Amount Ranges");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Test various amount sizes
    let amounts: &[(u32, &str)] = &[
        (1, "1_unit"),
        (10, "10_units"),
        (99, "99_units"),
        (100, "100_units"),
        (999, "999_units"),
        (1000, "1k_units"),
        (10000, "10k_units"),
        (100000, "100k_units"),
        (1000000, "1M_units"),
        (u32::MAX, "max_units"),
    ];

    for (amount, label) in amounts {
        group.bench_with_input(BenchmarkId::new("amount", label), amount, |b, &a| {
            b.iter(|| black_box(a).to_currency(&Locale::en))
        });
    }

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_representation_symbols(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Symbols");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Locales with different symbol types
    let symbol_types = [
        (Locale::en, "english_currency"),
        (Locale::de, "german_currency"),
        (Locale::ja, "japanese_currency"),
        (Locale::ar, "arabic_currency"),
        (Locale::pt, "portuguese_currency"),
        (Locale::ru, "russian_currency"),
        (Locale::fr, "french_currency"),
    ];

    for (locale, label) in &symbol_types {
        group.bench_function(format!("symbol_{}", label), |b| {
            b.iter(|| black_box(1234).to_currency(&locale))
        });
    }

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_formatting_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Formatting Patterns");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    let amount = 1234u32;

    // Different grouping patterns
    group.bench_function("format_en_grouping", |b| {
        b.iter(|| black_box(amount).to_currency(&Locale::en))
    });

    group.bench_function("format_de_grouping", |b| {
        b.iter(|| black_box(amount).to_currency(&Locale::de))
    });

    group.bench_function("format_fr_grouping", |b| {
        b.iter(|| black_box(amount).to_currency(&Locale::fr))
    });

    // Test with zero
    group.bench_function("format_zero_amount", |b| {
        b.iter(|| black_box(0u32).to_currency(&Locale::en))
    });

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Batch Operations");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(5));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Format same amount across 5 locales
    group.bench_function("batch_same_amount_5_locales", |b| {
        let locales = [Locale::en, Locale::de, Locale::fr, Locale::ja, Locale::ar];
        b.iter(|| {
            for &loc in &locales {
                let _ = black_box(100u32).to_currency(&loc);
            }
        })
    });

    // Format multiple amounts in single locale
    group.bench_function("batch_multiple_amounts_en", |b| {
        let amounts = [1u32, 10, 100, 1000, 10000, 100000, 1000000];
        b.iter(|| {
            for &amt in &amounts {
                let _ = black_box(amt).to_currency(&Locale::en);
            }
        })
    });

    // Format multiple amounts across locales (real-world scenario)
    group.bench_function("batch_invoice_style", |b| {
        let locales = [Locale::en, Locale::de, Locale::fr, Locale::ja];
        let amounts = [100u32, 250, 500, 1000, 5000];
        b.iter(|| {
            for &loc in &locales {
                for &amt in &amounts {
                    let _ = black_box(amt).to_currency(&loc);
                }
            }
        })
    });

    group.finish();
}

#[cfg(feature = "currency")]
fn bench_currency_large_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Large Scale");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(5));
    group.warm_up_time(std::time::Duration::from_secs(1));

    // Format across many locales (all available)
    group.bench_function("all_locales_100_units", |b| {
        b.iter(|| {
            let mut count = 0;
            for &locale_str in &locale_rs::AVAILABLE_LOCALES[..50] {
                if let Ok(locale) = Locale::from_str(black_box(locale_str)) {
                    let _ = black_box(100u32).to_currency(&locale);
                    count += 1;
                }
            }
            count
        })
    });

    group.finish();
}

#[cfg(feature = "currency")]
criterion_group!(
    benches,
    bench_currency_basic,
    bench_currency_multilingual,
    bench_currency_regional_variants,
    bench_currency_amounts,
    bench_currency_representation_symbols,
    bench_currency_formatting_patterns,
    bench_currency_batch_operations,
    bench_currency_large_scale,
);

#[cfg(feature = "currency")]
criterion_main!(benches);

#[cfg(not(feature = "currency"))]
fn main() {
    println!("Error: currency feature is required for this benchmark");
    println!("Run with: cargo bench --bench currency_bench --features currency");
}
