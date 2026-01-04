#[cfg(feature = "nums")]
fn main() {
    use locale_rs;
    use locale_rs::num_formats::ToFormattedString;
    let locale = locale_rs::Locale::ar_EG;
    for i in 0u32..9 {
        println!("{}", i.to_formatted_string(&locale))
    }
}

#[cfg(not(feature = "nums"))]
fn main() {
    panic!("Feature nums is not enabled")
}
