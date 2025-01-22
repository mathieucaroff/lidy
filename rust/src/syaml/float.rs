pub fn must_parse_float(value: &str) -> f64 {
    match value.parse::<f64>() {
        Ok(value) => value,
        Err(e) => match value.to_lowercase().as_ref() {
            ".inf" => f64::INFINITY,
            "-.inf" => f64::NEG_INFINITY,
            ".nan" => f64::NAN,
            _ => panic!("failed to parse Yaml Real ({value}) into Rust f64 float: {e}"),
        },
    }
}
