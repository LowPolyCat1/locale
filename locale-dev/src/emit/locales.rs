//! Emits `data/locales.rs`: the `Locale` enum, its identifiers, the lookup
//! map used by `FromStr` and the parent table.

use super::RustFile;
use crate::cldr::Cldr;
use crate::error::Result;
use crate::sanitize_variant;
use proc_macro2::Ident;
use quote::{format_ident, quote};

fn variant(name: &str) -> Ident {
    format_ident!("{}", sanitize_variant(name))
}

pub fn emit(cldr: &Cldr, cldr_version: &str) -> Result<RustFile> {
    let mut file = RustFile::new(
        cldr_version,
        "Locale identifiers and the parent locale table.",
    );
    let names: Vec<&str> = cldr.locales.iter().map(|l| l.name.as_str()).collect();
    let count = proc_macro2::Literal::usize_unsuffixed(names.len());

    let variants = names.iter().map(|name| {
        let v = variant(name);
        let doc = format!("`{name}`");
        quote! { #[doc = #doc] #v }
    });

    let mut map = phf_codegen::Map::new();
    for name in &names {
        map.entry(
            name.to_lowercase(),
            format!("Locale::{}", sanitize_variant(name)),
        );
    }
    let map: syn::Expr = syn::parse_str(&map.build().to_string())?;

    let enum_doc = format!(
        "A CLDR locale. There is one variant per locale of CLDR {cldr_version}; \
         variant names are the identifiers with `-` replaced by `_`."
    );

    file.items(quote! {
        /// The CLDR release this data was generated from.
        pub const CLDR_VERSION: &str = #cldr_version;

        /// Identifiers of all locales, in the order of the [`Locale`] variants.
        pub const AVAILABLE_LOCALES: [&str; #count] = [#(#names),*];

        #[doc = #enum_doc]
        #[cfg_attr(feature = "strum", derive(strum_macros::EnumIter))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        #[allow(non_camel_case_types)]
        pub enum Locale {
            #(#variants,)*
        }

        /// Lowercase identifier to locale.
        pub(crate) static LOCALE_MAP: phf::Map<&'static str, Locale> = #map;
    })?;

    let rows: Vec<(&str, _)> = cldr
        .locales
        .iter()
        .map(|l| {
            let parent = match l.parent {
                Some(p) => {
                    let p = variant(&cldr.locales[p].name);
                    quote!(Some(Locale::#p))
                }
                None => quote!(None),
            };
            (l.name.as_str(), parent)
        })
        .collect();
    file.locale_table(quote!(pub(crate)), "PARENTS", quote!(Option<Locale>), &rows);

    Ok(file)
}
