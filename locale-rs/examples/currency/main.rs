#[cfg(feature = "currency")]
fn main() {
    use locale_rs;
    use locale_rs::currency_formats::ToCurrencyString;
    let locale = locale_rs::Locale::en;
    for i in 0u32..10 {
        println!("{}", i.to_currency(&locale))
    }
    let locale = locale_rs::Locale::de;
    for i in 0u32..10 {
        println!("{}", i.to_currency(&locale))
    }
}

#[cfg(not(feature = "currency"))]
fn main() {
    panic!("Feature currency is not enabled")
}
