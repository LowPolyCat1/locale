mod test_locales;
#[cfg(feature = "nums")]
mod test_num_formatting;
#[cfg(feature = "strum")]
mod test_strum;

#[cfg(feature = "datetime")]
mod test_datetime_formatting;

#[cfg(feature = "currency")]
mod test_currency_formatting;
