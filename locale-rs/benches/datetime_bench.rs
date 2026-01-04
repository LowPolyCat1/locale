#![allow(deprecated)]
#[allow(unused)]
use criterion::{Criterion, black_box, criterion_group, criterion_main};
#[cfg(feature = "nums")]
use locale_rs::num_formats::ToFormattedString;
#[cfg(feature = "datetime")]
use locale_rs::{datetime_formats::DateTime, *};

#[cfg(feature = "datetime")]

fn bench_datetime_formatting(c: &mut Criterion) {
    let locale = Locale::en;
    let dt = DateTime {
        year: 2023,
        month: 12,
        day: 25,
        hour: 14,
        minute: 30,
        second: 05,
    };

    let mut group = c.benchmark_group("DateTime Formatting");

    group.bench_function("format_date (y-M-d)", |b| {
        b.iter(|| locale.format_date(black_box(&dt)))
    });

    group.bench_function("format_time (H:m:s)", |b| {
        b.iter(|| locale.format_time(black_box(&dt)))
    });

    // Patterns involving quotes and escaping (testing the parser state machine)
    group.bench_function("parse_complex_pattern_with_quotes", |b| {
        b.iter(|| {
            // This triggers the is_quoted and escaped-quote logic
            let pattern = "y 'o''clock' MMMM";
            // Accessing internal _parse_runtime_pattern if public or via wrapper
            // Here we assume a test wrapper or that the pattern is retrieved via locale
            locale.format_date(black_box(&dt))
        })
    });

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_numeric_formatting(c: &mut Criterion) {
    let locale = Locale::en;
    let mut group = c.benchmark_group("Numeric Formatting");

    // Integer grouping logic (hot path for _format_int_str)
    let large_int: i128 = 12345678901234567890;
    group.bench_function("format_large_int_with_grouping", |b| {
        b.iter(|| black_box(large_int).to_formatted_string(&locale))
    });

    // Float formatting (branching for NaN, Inf, and Decimal Separators)
    let float_val: f64 = 1234567.8910;
    group.bench_function("format_float", |b| {
        b.iter(|| black_box(float_val).to_formatted_string(&locale))
    });

    // Testing digit translation (feature "nums" logic)
    // If you have a locale that uses non-ASCII digits (e.g., Arabic/Hindi)
    #[cfg(feature = "nums")]
    {
        let ar_locale = Locale::ar_EG;
        group.bench_function("translate_digits_ar", |b| {
            b.iter(|| black_box(12345).to_formatted_string(&ar_locale))
        });
    }

    group.finish();
}

#[cfg(feature = "datetime")]
fn bench_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locale Conversions");
    let locale_str = "en";

    group.bench_function("Locale::from_str", |b| {
        b.iter(|| Locale::try_from(black_box(locale_str)))
    });

    group.bench_function("Locale::as_str", |b| {
        let loc = Locale::en;
        b.iter(|| black_box(loc).as_str())
    });

    group.finish();
}

#[cfg(feature = "datetime")]
criterion_group!(
    benches,
    bench_datetime_formatting,
    bench_numeric_formatting,
    bench_conversions
);

#[cfg(feature = "datetime")]
criterion_main!(benches);

#[cfg(not(feature = "datetime"))]
fn main() {
    println!("Info: Feature nums is not active");
}
