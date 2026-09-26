//! Emits `data/currency.rs`: pre-parsed currency patterns, default
//! currencies, currency symbols and fraction digits.
//!
//! Symbols are stored as overrides: a locale only lists the symbols that
//! differ from what its parent resolves to, and `locale-rs` walks the
//! fallback chain. This mirrors CLDR inheritance and keeps the table small.

use super::numbers::grouping_tokens;
use super::{Pool, RustFile};
use crate::cldr::{Cldr, is_currency_code};
use crate::error::{Error, Result};
use crate::patterns::parse_currency_pattern;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::collections::BTreeSet;

fn currency(code: &str) -> TokenStream {
    let bytes = Literal::byte_string(code.as_bytes());
    quote!(Currency(*#bytes))
}

/// The symbol `locale-rs` resolves for `code` in the locale at `index`: the
/// first symbol along the parent chain, else the code itself.
pub fn resolve_symbol<'a>(
    cldr: &'a Cldr,
    overrides: &'a [Vec<(String, String)>],
    index: usize,
    code: &'a str,
) -> &'a str {
    std::iter::successors(Some(index), |&i| cldr.locales[i].parent)
        .find_map(|i| {
            overrides[i]
                .iter()
                .find(|(c, _)| c == code)
                .map(|(_, s)| s.as_str())
        })
        .unwrap_or(code)
}

/// Per locale, the `(code, symbol)` pairs that differ from the parent's
/// resolution, sorted by code.
pub fn symbol_overrides(cldr: &Cldr) -> Vec<Vec<(String, String)>> {
    let codes: BTreeSet<&str> = cldr
        .locales
        .iter()
        .flat_map(|l| l.currency_symbols.keys().map(String::as_str))
        .collect();
    let expected = |i: usize, code: &str| -> String {
        cldr.locales[i]
            .currency_symbols
            .get(code)
            .cloned()
            .unwrap_or_else(|| code.to_string())
    };
    cldr.locales
        .iter()
        .enumerate()
        .map(|(i, l)| {
            codes
                .iter()
                .filter_map(|&code| {
                    let own = expected(i, code);
                    let inherited = match l.parent {
                        Some(p) => expected(p, code),
                        None => code.to_string(),
                    };
                    (own != inherited).then(|| (code.to_string(), own))
                })
                .collect()
        })
        .collect()
}

/// Checks that resolving through the overrides yields every symbol CLDR
/// lists for a locale.
fn verify_symbols(cldr: &Cldr, overrides: &[Vec<(String, String)>]) -> Result<()> {
    for (i, l) in cldr.locales.iter().enumerate() {
        for (code, symbol) in &l.currency_symbols {
            let resolved = resolve_symbol(cldr, overrides, i, code);
            if resolved != symbol {
                return Err(Error::Cldr(format!(
                    "{}: {code} resolves to {resolved:?} instead of {symbol:?}",
                    l.name
                )));
            }
        }
    }
    Ok(())
}

pub fn emit(cldr: &Cldr, cldr_version: &str) -> Result<RustFile> {
    let mut file = RustFile::new(
        cldr_version,
        "Currency patterns, symbols and defaults of every locale.",
    );
    let mut patterns = Pool::new("PATTERN", quote!(CurrencyPattern));
    let mut symbol_lists = Vec::new();
    let mut symbol_names = std::collections::HashMap::new();

    let overrides = symbol_overrides(cldr);
    verify_symbols(cldr, &overrides)?;
    let mut pattern_rows = Vec::new();
    let mut default_rows = Vec::new();
    let mut symbol_rows = Vec::new();

    for (l, symbols) in cldr.locales.iter().zip(&overrides) {
        let name = l.name.as_str();

        let p = parse_currency_pattern(&l.currency_pattern, &l.numbers.minus_sign);
        let (pp, ps, np, ns) = (
            &p.positive_prefix,
            &p.positive_suffix,
            &p.negative_prefix,
            &p.negative_suffix,
        );
        let grouping = grouping_tokens(p.grouping);
        let pattern = patterns.intern(quote! {
            CurrencyPattern {
                positive_prefix: #pp,
                positive_suffix: #ps,
                negative_prefix: #np,
                negative_suffix: #ns,
                grouping: #grouping,
            }
        });
        pattern_rows.push((name, quote!(&#pattern)));
        default_rows.push((name, currency(&l.default_currency)));

        let symbols_row = if symbols.is_empty() {
            quote!(&[])
        } else {
            let entries = symbols.iter().map(|(code, symbol)| {
                let code = currency(code);
                quote!((#code, #symbol))
            });
            let list = quote!([#(#entries),*]);
            let key = list.to_string();
            let ident = symbol_names.entry(key).or_insert_with(|| {
                let ident = quote::format_ident!("SYMBOLS_{}", symbol_lists.len());
                let len = Literal::usize_unsuffixed(symbols.len());
                symbol_lists.push(quote!(static #ident: [(Currency, &str); #len] = #list;));
                ident
            });
            quote!(&#ident)
        };
        symbol_rows.push((name, symbols_row));
    }

    let fractions = cldr
        .fraction_digits
        .iter()
        .filter(|(code, _)| is_currency_code(code))
        .map(|(code, &digits)| {
            let code = currency(code);
            let digits = Literal::u8_unsuffixed(digits);
            quote!((#code, #digits))
        });
    let fraction_count = Literal::usize_unsuffixed(cldr.fraction_digits.len());

    let patterns = patterns.items();
    file.items(quote! {
        use crate::currency::{Currency, CurrencyPattern};
        use crate::data::Grouping;

        /// Currencies whose fraction digits are not 2, sorted by code.
        pub(crate) static FRACTION_DIGITS: [(Currency, u8); #fraction_count] = [#(#fractions),*];

        #patterns
        #(#symbol_lists)*
    })?;
    file.locale_table(
        quote!(pub(crate)),
        "CURRENCY_PATTERNS",
        quote!(&CurrencyPattern),
        &pattern_rows,
    );
    file.locale_table(
        quote!(pub(crate)),
        "DEFAULT_CURRENCIES",
        quote!(Currency),
        &default_rows,
    );
    file.locale_table(
        quote!(pub(crate)),
        "CURRENCY_SYMBOLS",
        quote!(&[(Currency, &str)]),
        &symbol_rows,
    );
    Ok(file)
}
