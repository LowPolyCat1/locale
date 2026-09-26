use locale_rs::{Locale, NumberFormatter};

fn main() {
    let ar = NumberFormatter::new(Locale::ar_EG);
    for i in 0u32..10 {
        println!("{}", ar.format(i));
    }
    println!("{}", ar.format(-1234567.89));
}
