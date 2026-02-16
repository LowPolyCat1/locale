use serde_json::Value;
use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

use crate::sanitize_variant;

pub fn run(
    zip_buffer: Vec<u8>,
    _asset_name: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_buffer))?;

    let mut locales = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.name().contains("/main/") && file.is_dir() {
            let parts: Vec<&str> = file.name().split('/').collect();
            if let Some(idx) = parts.iter().position(|&r| r == "main")
                && let Some(name) = parts.get(idx + 1)
                && !name.is_empty()
                && !locales.contains(&(*name).to_string())
            {
                locales.push((*name).to_string());
            }
        }
    }
    locales.sort();

    let mut months_wide_arms = String::new();
    let mut months_abbr_arms = String::new();
    let mut days_wide_arms = String::new();
    let mut date_format_arms = String::new();
    let mut time_format_arms = String::new();
    let mut am_pm_arms = String::new();

    for name in &locales {
        let var = sanitize_variant(name);
        let json_path = format!("cldr-dates-full/main/{}/ca-gregorian.json", name);

        if let Ok(mut file) = archive.by_name(&json_path) {
            let json: Value = serde_json::from_reader(&mut file)?;
            let greg = &json["main"][name]["dates"]["calendars"]["gregorian"];

            let m_wide = extract_indexed_months(&greg["months"]["format"]["wide"]);
            let m_abbr = extract_indexed_months(&greg["months"]["format"]["abbreviated"]);
            let d_wide = extract_days(&greg["days"]["format"]["wide"]);

            // Extract AM/PM markers
            let am = greg["dayPeriods"]["format"]["wide"]["am"]
                .as_str()
                .unwrap_or("AM");
            let pm = greg["dayPeriods"]["format"]["wide"]["pm"]
                .as_str()
                .unwrap_or("PM");

            let d_medium = greg["dateFormats"]["medium"].as_str().unwrap_or("y-MM-dd");
            let t_medium = greg["timeFormats"]["medium"].as_str().unwrap_or("HH:mm:ss");

            months_wide_arms.push_str(&format!("            Locale::{} => &{:?},\n", var, m_wide));
            months_abbr_arms.push_str(&format!("            Locale::{} => &{:?},\n", var, m_abbr));
            days_wide_arms.push_str(&format!("            Locale::{} => &{:?},\n", var, d_wide));
            am_pm_arms.push_str(&format!(
                "            Locale::{} => ({:?}, {:?}),\n",
                var, am, pm
            ));
            date_format_arms.push_str(&format!("            Locale::{} => {:?},\n", var, d_medium));
            time_format_arms.push_str(&format!("            Locale::{} => {:?},\n", var, t_medium));
        } else {
            months_wide_arms.push_str(&format!("            Locale::{} => &[\"1\",\"2\",\"3\",\"4\",\"5\",\"6\",\"7\",\"8\",\"9\",\"10\",\"11\",\"12\"],\n", var));
            months_abbr_arms.push_str(&format!("            Locale::{} => &[\"1\",\"2\",\"3\",\"4\",\"5\",\"6\",\"7\",\"8\",\"9\",\"10\",\"11\",\"12\"],\n", var));
            days_wide_arms.push_str(&format!("            Locale::{} => &[\"Sun\",\"Mon\",\"Tue\",\"Wed\",\"Thu\",\"Fri\",\"Sat\"],\n", var));
            am_pm_arms.push_str(&format!(
                "            Locale::{} => (\"AM\", \"PM\"),\n",
                var
            ));
            date_format_arms.push_str(&format!("            Locale::{} => \"y-MM-dd\",\n", var));
            time_format_arms.push_str(&format!("            Locale::{} => \"HH:mm:ss\",\n", var));
        }
    }

    let code = format!(
        r#"// Auto-generated. DO NOT EDIT.
use crate::locale::Locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {{
    pub year: i32,
    pub month: u32,  // 1-12
    pub day: u32,    // 1-31
    pub hour: u32,   // 0-23
    pub minute: u32, // 0-59
    pub second: u32, // 0-59
}}

impl Locale {{
    pub fn months_wide(&self) -> &'static [&'static str] {{
        match self {{ {months_wide_arms} }}
    }}

    pub fn months_abbreviated(&self) -> &'static [&'static str] {{
        match self {{ {months_abbr_arms} }}
    }}

    pub fn days_wide(&self) -> &'static [&'static str] {{
        match self {{ {days_wide_arms} }}
    }}

    fn am_pm(&self) -> (&'static str, &'static str) {{
        match self {{ {am_pm_arms} }}
    }}

    pub fn date_format_pattern(&self) -> &'static str {{
        match self {{ {date_format_arms} }}
    }}

    pub fn time_format_pattern(&self) -> &'static str {{
        match self {{ {time_format_arms} }}
    }}

    pub fn format_date(&self, dt: &DateTime) -> String {{
        self._parse_runtime_pattern(self.date_format_pattern(), dt)
    }}

    pub fn format_time(&self, dt: &DateTime) -> String {{
        self._parse_runtime_pattern(self.time_format_pattern(), dt)
    }}

    fn _parse_runtime_pattern(&self, pattern: &str, dt: &DateTime) -> String {{
        let mut result = String::new();
        let mut chars = pattern.chars().peekable();
        let mut is_quoted = false;

        while let Some(c) = chars.next() {{
            if c == '\'' {{
                if let Some(&'\'') = chars.peek() {{
                    result.push('\'');
                    chars.next();
                }} else {{
                    is_quoted = !is_quoted;
                }}
                continue;
            }}

            if is_quoted {{
                result.push(c);
                continue;
            }}

            let mut count = 1;
            while let Some(&next_c) = chars.peek() {{
                if next_c == c {{
                    count += 1;
                    chars.next();
                }} else {{
                    break;
                }}
            }}

            match c {{
                'y' => {{
                    let year_str = dt.year.to_string();
                    if count == 2 && year_str.len() > 2 {{
                        result.push_str(&year_str[year_str.len()-2..]);
                    }} else {{
                        result.push_str(&format!("{{:0width$}}", dt.year, width = count));
                    }}
                }},
                'M' => {{
                    match count {{
                        1 | 2 => result.push_str(&format!("{{:0width$}}", dt.month, width = count)),
                        3 => result.push_str(self.months_abbreviated().get((dt.month - 1) as usize).unwrap_or(&"")),
                        _ => result.push_str(self.months_wide().get((dt.month - 1) as usize).unwrap_or(&"")),
                    }}
                }},
                'd' => result.push_str(&format!("{{:0width$}}", dt.day, width = count)),
                'H' => result.push_str(&format!("{{:0width$}}", dt.hour, width = count)),
                'h' => {{
                    let h12 = if dt.hour % 12 == 0 {{ 12 }} else {{ dt.hour % 12 }};
                    result.push_str(&format!("{{:0width$}}", h12, width = count));
                }},
                'm' => result.push_str(&format!("{{:0width$}}", dt.minute, width = count)),
                's' => result.push_str(&format!("{{:0width$}}", dt.second, width = count)),
                'a' => {{
                    let (am, pm) = self.am_pm();
                    result.push_str(if dt.hour < 12 {{ am }} else {{ pm }});
                }},
                'E' => {{
                    let dow = self._calculate_weekday(dt.year, dt.month, dt.day);
                    result.push_str(self.days_wide().get(dow as usize).unwrap_or(&""));
                }},
                _ => {{
                    for _ in 0..count {{ result.push(c); }}
                }}
            }}
        }}

        #[cfg(feature = "nums")]
        {{ result = crate::num_formats::translate_digits(result, self); }}

        result
    }}

    fn _calculate_weekday(&self, y: i32, m: u32, d: u32) -> u32 {{
        // Sakamoto's algorithm (0=Sun, 1=Mon...)
        static T: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = y;
        if m < 3 {{ y -= 1; }}
        ( (y + y/4 - y/100 + y/400 + T[(m-1) as usize] as i32 + d as i32) % 7 ) as u32
    }}
}}
"#,
        months_wide_arms = months_wide_arms,
        months_abbr_arms = months_abbr_arms,
        days_wide_arms = days_wide_arms,
        am_pm_arms = am_pm_arms,
        date_format_arms = date_format_arms,
        time_format_arms = time_format_arms
    );

    fs::write(output_path, code)?;
    Ok(())
}

fn extract_indexed_months(obj: &Value) -> Vec<String> {
    let mut result = Vec::new();
    if let Some(map) = obj.as_object() {
        for i in 1..=12 {
            let val = map
                .get(&i.to_string())
                .and_then(|v| v.as_str())
                .unwrap_or("");
            result.push(val.to_string());
        }
    }
    result
}

fn extract_days(obj: &Value) -> Vec<String> {
    let keys = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    let mut result = Vec::new();
    if let Some(map) = obj.as_object() {
        for key in keys {
            let val = map.get(key).and_then(|v| v.as_str()).unwrap_or("");
            result.push(val.to_string());
        }
    }
    result
}
