#![allow(deprecated)]
#[cfg(feature = "nums")]
#[allow(unused)]
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
#[cfg(feature = "nums")]
use locale_rs::{Locale, num_formats::ToFormattedString};

#[cfg(feature = "nums")]
fn bench_integer_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Integer Formatting");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    let locales = [Locale::en, Locale::de, Locale::fr, Locale::ja, Locale::ar];

    // Small integers (common case)
    group.bench_function("i32_small", |b| {
        b.iter(|| black_box(42i32).to_formatted_string(&Locale::en))
    });

    group.bench_function("i32_small_multilingual", |b| {
        b.iter(|| {
            for &loc in &locales {
                let _ = black_box(42i32).to_formatted_string(&loc);
            }
        })
    });

    // Medium integers (common case)
    group.bench_function("i32_medium", |b| {
        b.iter(|| black_box(1234567i32).to_formatted_string(&Locale::en))
    });

    group.bench_function("i32_medium_multilingual", |b| {
        b.iter(|| {
            for &loc in &locales {
                let _ = black_box(1234567i32).to_formatted_string(&loc);
            }
        })
    });

    // Large integers
    group.bench_function("i64_large", |b| {
        b.iter(|| black_box(1234567890123i64).to_formatted_string(&Locale::en))
    });

    group.bench_function("i64_very_large", |b| {
        b.iter(|| black_box(9223372036854775807i64).to_formatted_string(&Locale::en))
    });

    // Special values
    group.bench_function("i32_zero", |b| {
        b.iter(|| black_box(0i32).to_formatted_string(&Locale::en))
    });

    group.bench_function("i32_negative_small", |b| {
        b.iter(|| black_box(-42i32).to_formatted_string(&Locale::en))
    });

    group.bench_function("i32_negative_large", |b| {
        b.iter(|| black_box(-1234567i32).to_formatted_string(&Locale::en))
    });

    group.bench_function("i64_negative_max", |b| {
        b.iter(|| black_box(i64::MIN).to_formatted_string(&Locale::en))
    });

    group.finish();
}

#[cfg(feature = "nums")]
fn bench_float_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Float Formatting");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    let locales = [Locale::en, Locale::de, Locale::fr, Locale::ja, Locale::ar];

    // Small decimals
    group.bench_function("f64_small", |b| {
        b.iter(|| black_box(3.14f64).to_formatted_string(&Locale::en))
    });

    // Medium decimals
    group.bench_function("f64_medium", |b| {
        b.iter(|| black_box(1234.5678f64).to_formatted_string(&Locale::en))
    });

    // Large decimals
    group.bench_function("f64_large", |b| {
        b.iter(|| black_box(1234567.89f64).to_formatted_string(&Locale::en))
    });

    // Very large decimals
    group.bench_function("f64_very_large", |b| {
        b.iter(|| black_box(1234567890.123456f64).to_formatted_string(&Locale::en))
    });

    // Special values
    group.bench_function("f64_zero", |b| {
        b.iter(|| black_box(0.0f64).to_formatted_string(&Locale::en))
    });

    group.bench_function("f64_negative", |b| {
        b.iter(|| black_box(-1234.5678f64).to_formatted_string(&Locale::en))
    });

    group.bench_function("f64_very_small", |b| {
        b.iter(|| black_box(0.0001f64).to_formatted_string(&Locale::en))
    });

    // Multilingual float formatting
    group.bench_function("f64_multilingual", |b| {
        b.iter(|| {
            for &loc in &locales {
                let _ = black_box(1234567.891f64).to_formatted_string(&loc);
            }
        })
    });

    group.finish();
}

#[cfg(feature = "nums")]
fn bench_digit_grouping(c: &mut Criterion) {
    let mut group = c.benchmark_group("Digit Grouping");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Numbers with increasing digit counts to test grouping performance
    let numbers: &[(i64, &str)] = &[
        (100, "3-digits"),
        (1000, "4-digits"),
        (10000, "5-digits"),
        (100000, "6-digits"),
        (1000000, "7-digits"),
        (10000000, "8-digits"),
        (100000000, "9-digits"),
        (1000000000, "10-digits"),
    ];

    for (num, label) in numbers {
        group.bench_with_input(BenchmarkId::new("grouping", label), num, |b, &n| {
            b.iter(|| black_box(n).to_formatted_string(&Locale::en))
        });
    }

    // Compare grouping behavior across locales with different separators
    group.bench_function("grouping_en_comma", |b| {
        b.iter(|| black_box(1234567890i64).to_formatted_string(&Locale::en))
    });

    group.bench_function("grouping_de_period", |b| {
        b.iter(|| black_box(1234567890i64).to_formatted_string(&Locale::de))
    });

    group.bench_function("grouping_fr_space", |b| {
        b.iter(|| black_box(1234567890i64).to_formatted_string(&Locale::fr))
    });

    group.finish();
}

#[cfg(feature = "nums")]
fn bench_native_digits(c: &mut Criterion) {
    let mut group = c.benchmark_group("Native Digit Systems");
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(3));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Arabic-Indic digits
    group.bench_function("arabic_indic_numerals", |b| {
        b.iter(|| black_box(123456).to_formatted_string(&Locale::ar_EG))
    });

    // Test various Arabic locales
    let arabic_locales = [
        Locale::ar,
        Locale::ar_SA,
        Locale::ar_EG,
        Locale::ar_AE,
        Locale::ar_BH,
    ];

    for &locale in &arabic_locales {
        group.bench_function(format!("arabic_numerals_{}", locale.as_str()), |b| {
            b.iter(|| black_box(12345).to_formatted_string(&locale))
        });
    }

    // Extended Arabic-Indic digits
    group.bench_function("extended_arabic_indic", |b| {
        b.iter(|| black_box(789456123).to_formatted_string(&Locale::ar_EG))
    });

    group.finish();
}

#[cfg(feature = "nums")]
fn bench_large_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Number Formatting");
    group.sample_size(50); // Smaller sample for large numbers
    group.measurement_time(std::time::Duration::from_secs(5));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Progressively larger numbers
    group.bench_function("i128_huge", |b| {
        b.iter(|| black_box(12345678901234567890123456789i128).to_formatted_string(&Locale::en))
    });

    // Very long decimal numbers
    group.bench_function("f64_many_decimals", |b| {
        b.iter(|| black_box(123456789.123456789f64).to_formatted_string(&Locale::en))
    });

    group.finish();
}

#[cfg(feature = "nums")]
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Numeric Operations");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(5));
    group.warm_up_time(std::time::Duration::from_millis(500));

    // Format same value across all common locales
    group.bench_function("format_single_value_5_locales", |b| {
        let locales = [Locale::en, Locale::de, Locale::fr, Locale::ja, Locale::ar];
        b.iter(|| {
            for &loc in &locales {
                let _ = black_box(1234567).to_formatted_string(&loc);
            }
        })
    });

    // Format multiple values in single locale
    group.bench_function("format_10_values_single_locale", |b| {
        let values = [
            1,
            10,
            100,
            1000,
            10000,
            100000,
            1000000,
            10000000,
            100000000,
            1000000000i64,
        ];
        b.iter(|| {
            for &val in &values {
                let _ = black_box(val).to_formatted_string(&Locale::en);
            }
        })
    });

    // Format range of values across multiple locales
    group.bench_function("format_mixed_5_locales_10_values", |b| {
        let locales = [Locale::en, Locale::de, Locale::fr, Locale::ja, Locale::ar];
        let values = [
            1,
            10,
            100,
            1000,
            10000,
            100000,
            1000000,
            10000000,
            100000000,
            1000000000i64,
        ];
        b.iter(|| {
            for &loc in &locales {
                for &val in &values {
                    let _ = black_box(val).to_formatted_string(&loc);
                }
            }
        })
    });

    group.finish();
}

#[cfg(feature = "nums")]
criterion_group!(
    benches,
    bench_integer_formatting,
    bench_float_formatting,
    bench_digit_grouping,
    bench_native_digits,
    bench_large_numbers,
    bench_batch_operations,
);

#[cfg(feature = "nums")]
criterion_main!(benches);

#[cfg(not(feature = "nums"))]
fn main() {
    println!("Error: nums feature is required for this benchmark");
    println!("Run with: cargo bench --bench num_bench --features nums");
}
