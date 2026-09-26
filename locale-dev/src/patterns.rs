//! Parsers for CLDR number and date patterns.
//!
//! Patterns are parsed here, at generation time, so that `locale-rs` only
//! walks pre-parsed structures at runtime.

/// Digit grouping of a number pattern; a primary size of 0 means none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Grouping {
    pub primary: u8,
    pub secondary: u8,
}

/// Grouping sizes of the integer part of a number pattern.
///
/// `#,##0.###` groups by 3, `#,##,##0.###` by 3 and then 2, `0.###` not at
/// all.
pub fn grouping(pattern: &str) -> Grouping {
    let integer = pattern.split('.').next().unwrap_or(pattern);
    let clean: String = integer
        .chars()
        .filter(|c| matches!(c, '#' | '0' | ','))
        .collect();
    let groups: Vec<&str> = clean.split(',').collect();
    let size = |g: &str| u8::try_from(g.len()).unwrap_or(u8::MAX);
    match groups.as_slice() {
        [] | [_] => Grouping {
            primary: 0,
            secondary: 0,
        },
        [.., primary] if groups.len() == 2 => Grouping {
            primary: size(primary),
            secondary: size(primary),
        },
        [.., secondary, primary] => Grouping {
            primary: size(primary),
            secondary: size(secondary),
        },
    }
}

/// One element of a date pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DateToken {
    Literal(String),
    /// A supported field letter and how often it is repeated.
    Field(char, u8),
}

/// Fields `locale-rs` formats. Any other letter is kept as literal text.
const DATE_FIELDS: &[char] = &['y', 'M', 'd', 'H', 'h', 'm', 's', 'a', 'E'];

/// Splits a date pattern into fields and literal text.
///
/// Text in single quotes is literal and `''` is a literal quote, both inside
/// and outside of quoted text.
pub fn parse_date_pattern(pattern: &str) -> Vec<DateToken> {
    let mut tokens: Vec<DateToken> = Vec::new();
    let push_literal = |tokens: &mut Vec<DateToken>, text: &str| {
        if let Some(DateToken::Literal(last)) = tokens.last_mut() {
            last.push_str(text);
        } else {
            tokens.push(DateToken::Literal(text.to_string()));
        }
    };

    let mut chars = pattern.chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        if c == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                push_literal(&mut tokens, "'");
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if quoted {
            push_literal(&mut tokens, c.encode_utf8(&mut [0; 4]));
            continue;
        }

        let mut count: u8 = 1;
        while chars.peek() == Some(&c) {
            chars.next();
            count = count.saturating_add(1);
        }
        if DATE_FIELDS.contains(&c) {
            tokens.push(DateToken::Field(c, count));
        } else {
            push_literal(&mut tokens, &c.to_string().repeat(count.into()));
        }
    }
    tokens
}

/// A currency pattern split around its number. `¤` in an affix is the
/// currency symbol; negative affixes contain the locale's minus sign.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyPattern {
    pub positive_prefix: String,
    pub positive_suffix: String,
    pub negative_prefix: String,
    pub negative_suffix: String,
    pub grouping: Grouping,
}

/// Parses `positive[;negative]`. Without an explicit negative subpattern
/// the negative form is the positive one prefixed with `minus_sign`.
pub fn parse_currency_pattern(pattern: &str, minus_sign: &str) -> CurrencyPattern {
    let (positive, negative) = split_unquoted(pattern, ';');
    let (pos_prefix, number, pos_suffix) = split_number(positive);
    let positive_prefix = unescape_affix(pos_prefix, minus_sign);
    let positive_suffix = unescape_affix(pos_suffix, minus_sign);
    let (negative_prefix, negative_suffix) = match negative {
        Some(negative) => {
            let (prefix, _, suffix) = split_number(negative);
            (
                unescape_affix(prefix, minus_sign),
                unescape_affix(suffix, minus_sign),
            )
        }
        None => (
            format!("{minus_sign}{positive_prefix}"),
            positive_suffix.clone(),
        ),
    };
    CurrencyPattern {
        positive_prefix,
        positive_suffix,
        negative_prefix,
        negative_suffix,
        grouping: grouping(number),
    }
}

fn is_number_char(c: char) -> bool {
    matches!(c, '#' | '0'..='9' | ',' | '.' | '@')
}

/// Splits at the first `sep` outside of quotes.
fn split_unquoted(pattern: &str, sep: char) -> (&str, Option<&str>) {
    let mut quoted = false;
    for (i, c) in pattern.char_indices() {
        match c {
            '\'' => quoted = !quoted,
            c if c == sep && !quoted => return (&pattern[..i], Some(&pattern[i + 1..])),
            _ => {}
        }
    }
    (pattern, None)
}

/// Splits a subpattern into prefix, number and suffix. The number is the
/// span from the first to the last unquoted number character.
fn split_number(pattern: &str) -> (&str, &str, &str) {
    let mut quoted = false;
    let mut span: Option<(usize, usize)> = None;
    for (i, c) in pattern.char_indices() {
        if c == '\'' {
            quoted = !quoted;
        } else if !quoted && is_number_char(c) {
            let end = i + c.len_utf8();
            span = Some(span.map_or((i, end), |(start, _)| (start, end)));
        }
    }
    match span {
        Some((start, end)) => (&pattern[..start], &pattern[start..end], &pattern[end..]),
        None => (pattern, "", ""),
    }
}

/// Resolves quoting in an affix and replaces the unquoted `-` with the
/// locale's minus sign.
fn unescape_affix(affix: &str, minus_sign: &str) -> String {
    let mut out = String::new();
    let mut chars = affix.chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '\'' if chars.peek() == Some(&'\'') => {
                chars.next();
                out.push('\'');
            }
            '\'' => quoted = !quoted,
            '-' if !quoted => out.push_str(minus_sign),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use DateToken::{Field, Literal};

    fn g(primary: u8, secondary: u8) -> Grouping {
        Grouping { primary, secondary }
    }

    #[test]
    fn grouping_sizes() {
        assert_eq!(grouping("#,##0.###"), g(3, 3));
        assert_eq!(grouping("#,##,##0.###"), g(3, 2));
        assert_eq!(grouping("¤#,##0.00"), g(3, 3));
        assert_eq!(grouping("#,#0.###"), g(2, 2));
        assert_eq!(grouping("0.###"), g(0, 0));
        assert_eq!(grouping(""), g(0, 0));
    }

    #[test]
    fn date_fields_and_literals() {
        assert_eq!(
            parse_date_pattern("MMM d, y"),
            [
                Field('M', 3),
                Literal(" ".into()),
                Field('d', 1),
                Literal(", ".into()),
                Field('y', 1)
            ]
        );
        assert_eq!(
            parse_date_pattern("y年M月d日"),
            [
                Field('y', 1),
                Literal("年".into()),
                Field('M', 1),
                Literal("月".into()),
                Field('d', 1),
                Literal("日".into())
            ]
        );
    }

    #[test]
    fn date_quotes() {
        assert_eq!(
            parse_date_pattern("d 'de' MMMM"),
            [Field('d', 1), Literal(" de ".into()), Field('M', 4)]
        );
        assert_eq!(
            parse_date_pattern("h 'o''clock' a"),
            [Field('h', 1), Literal(" o'clock ".into()), Field('a', 1)]
        );
        assert_eq!(parse_date_pattern("''"), [Literal("'".into())]);
    }

    #[test]
    fn unsupported_letters_stay_literal() {
        assert_eq!(
            parse_date_pattern("GGG y"),
            [Literal("GGG ".into()), Field('y', 1)]
        );
    }

    #[test]
    fn currency_implicit_negative() {
        let p = parse_currency_pattern("#,##0.00\u{a0}¤", "-");
        assert_eq!(p.positive_prefix, "");
        assert_eq!(p.positive_suffix, "\u{a0}¤");
        assert_eq!(p.negative_prefix, "-");
        assert_eq!(p.negative_suffix, "\u{a0}¤");
        assert_eq!(p.grouping, g(3, 3));
    }

    #[test]
    fn currency_explicit_negative_uses_locale_minus() {
        let p = parse_currency_pattern("¤\u{a0}#,##0.00;¤-#,##0.00", "\u{2212}");
        assert_eq!(p.positive_prefix, "¤\u{a0}");
        assert_eq!(p.negative_prefix, "¤\u{2212}");
        assert_eq!(p.negative_suffix, "");
    }

    #[test]
    fn currency_indian_grouping_is_not_mangled() {
        let p = parse_currency_pattern("¤\u{a0}#,##,##0.00", "-");
        assert_eq!(p.positive_prefix, "¤\u{a0}");
        assert_eq!(p.positive_suffix, "");
        assert_eq!(p.grouping, g(3, 2));
    }

    #[test]
    fn currency_quoted_affix() {
        let p = parse_currency_pattern("'-'¤#,##0.00", "−");
        assert_eq!(p.positive_prefix, "-¤");
        assert_eq!(p.negative_prefix, "−-¤");
    }
}
