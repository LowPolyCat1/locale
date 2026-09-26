//! Emits `data/dates.rs`: Gregorian names and pre-parsed medium patterns.

use super::{Pool, RustFile};
use crate::cldr::Cldr;
use crate::error::Result;
use crate::patterns::{DateToken, parse_date_pattern};
use proc_macro2::{Literal, TokenStream};
use quote::quote;

/// The `DatePart`s of a pattern.
pub fn parts_tokens(pattern: &str) -> TokenStream {
    let parts = parse_date_pattern(pattern)
        .into_iter()
        .map(|token| match token {
            DateToken::Literal(text) => quote!(DatePart::Literal(#text)),
            DateToken::Field(c, width) => {
                let width = Literal::u8_unsuffixed(width);
                match c {
                    'y' => quote!(DatePart::Year(#width)),
                    'M' => quote!(DatePart::Month(#width)),
                    'd' => quote!(DatePart::Day(#width)),
                    'H' => quote!(DatePart::Hour24(#width)),
                    'h' => quote!(DatePart::Hour12(#width)),
                    'm' => quote!(DatePart::Minute(#width)),
                    's' => quote!(DatePart::Second(#width)),
                    'a' => quote!(DatePart::DayPeriod),
                    'E' => quote!(DatePart::Weekday),
                    other => unreachable!("`{other}` is not a supported date field"),
                }
            }
        });
    quote!(&[#(#parts),*])
}

pub fn emit(cldr: &Cldr, cldr_version: &str) -> Result<RustFile> {
    let mut file = RustFile::new(cldr_version, "Gregorian calendar data of every locale.");
    let mut months = Pool::new("MONTHS", quote!([&str; 12]));
    let mut weekdays = Pool::new("WEEKDAYS", quote!([&str; 7]));
    let mut patterns = Pool::new("PATTERN", quote!(DatePattern));
    let mut symbols = Pool::new("DATES", quote!(DateSymbols));

    let mut pattern = |source: &str| {
        let parts = parts_tokens(source);
        patterns.intern(quote!(DatePattern { source: #source, parts: #parts }))
    };

    let rows: Vec<(&str, TokenStream)> = cldr
        .locales
        .iter()
        .map(|l| {
            let d = &l.dates;
            let (wide, abbreviated, days) =
                (&d.months_wide, &d.months_abbreviated, &d.weekdays_wide);
            let wide = months.intern(quote!([#(#wide),*]));
            let abbreviated = months.intern(quote!([#(#abbreviated),*]));
            let days = weekdays.intern(quote!([#(#days),*]));
            let date = pattern(&d.date_pattern);
            let time = pattern(&d.time_pattern);
            let (am, pm) = (&d.am, &d.pm);
            let name = symbols.intern(quote! {
                DateSymbols {
                    months_wide: &#wide,
                    months_abbreviated: &#abbreviated,
                    weekdays_wide: &#days,
                    am: #am,
                    pm: #pm,
                    date_pattern: &#date,
                    time_pattern: &#time,
                }
            });
            (l.name.as_str(), quote!(&#name))
        })
        .collect();

    let items = [
        months.items(),
        weekdays.items(),
        patterns.items(),
        symbols.items(),
    ];
    file.items(quote! {
        use crate::datetime::{DatePart, DatePattern, DateSymbols};
        #(#items)*
    })?;
    file.locale_table(
        quote!(pub(crate)),
        "DATE_SYMBOLS",
        quote!(&DateSymbols),
        &rows,
    );
    Ok(file)
}
