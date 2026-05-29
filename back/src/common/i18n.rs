pub const DEFAULT_FALLBACK_CHAIN: [&str; 2] = ["en", "fr"];

pub fn locale_chain(requested: Option<&str>) -> Vec<String> {
    let mut result = Vec::new();

    if let Some(locale) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        result.push(locale.to_lowercase());
    }

    for fallback in DEFAULT_FALLBACK_CHAIN {
        if !result.iter().any(|value| value == fallback) {
            result.push(fallback.to_string());
        }
    }

    result
}
