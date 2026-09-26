//! Emits `data/numbers.rs`: the `NumberSymbols` of every locale.

use super::{Pool, RustFile};
use crate::cldr::Cldr;
use crate::error::Result;
use crate::patterns::{Grouping, grouping};
use proc_macro2::{Literal, TokenStream};
use quote::quote;

/// A runtime `Grouping`: the sizes from a pattern plus the locale's
/// minimum grouping digits.
pub fn grouping_tokens(g: Grouping, min_grouping_digits: u8) -> TokenStream {
    let primary = Literal::u8_unsuffixed(g.primary);
    let secondary = Literal::u8_unsuffixed(g.secondary);
    let min = Literal::u8_unsuffixed(min_grouping_digits);
    quote!(Grouping { primary: #primary, secondary: #secondary, min_grouping_digits: #min })
}

pub fn emit(cldr: &Cldr, cldr_version: &str) -> Result<RustFile> {
    let mut file = RustFile::new(cldr_version, "Number symbols of every locale.");
    let mut digits = Pool::new("DIGITS", quote!([char; 10]));
    let mut symbols = Pool::new("NUMBERS", quote!(NumberSymbols));

    let rows: Vec<(&str, TokenStream)> = cldr
        .locales
        .iter()
        .map(|l| {
            let n = &l.numbers;
            let digits = match n.digits {
                Some(d) => {
                    let name = digits.intern(quote!([#(#d),*]));
                    quote!(Some(&#name))
                }
                None => quote!(None),
            };
            let (decimal, group, minus) = (&n.decimal, &n.group, &n.minus_sign);
            let grouping = grouping_tokens(grouping(&n.decimal_pattern), n.min_grouping_digits);
            let name = symbols.intern(quote! {
                NumberSymbols {
                    decimal: #decimal,
                    group: #group,
                    minus_sign: #minus,
                    grouping: #grouping,
                    digits: #digits,
                }
            });
            (l.name.as_str(), quote!(&#name))
        })
        .collect();

    let (digits, symbols) = (digits.items(), symbols.items());
    file.items(quote! {
        use crate::data::{Grouping, NumberSymbols};
        #digits
        #symbols
    })?;
    file.locale_table(
        quote!(pub(crate)),
        "NUMBER_SYMBOLS",
        quote!(&NumberSymbols),
        &rows,
    );
    Ok(file)
}
