//! JSON utilities.

use serde_json::{Map, Number, Value};

/// `SortByPropertyName`: re-emits `json` indented with every object's properties sorted by name.
/// Numbers are rewritten as doubles (`1.0` prints as `1`), as `WriteNumberValue(GetDouble())` did.
/// Names sort with the culture comparer (`OrderBy(prop => prop.Name)`).
pub fn sort_by_property_name(json: &str) -> serde_json::Result<String> {
    let value: Value = serde_json::from_str(json)?;
    serde_json::to_string_pretty(&sorted(value))
}

fn sorted(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| muo_primitives::dotnet::string_compare(&a.0, &b.0)); // stable, as OrderBy
            Value::Object(
                entries
                    .into_iter()
                    .map(|(k, v)| (k, sorted(v)))
                    .collect::<Map<_, _>>(),
            )
        }
        Value::Array(a) => Value::Array(a.into_iter().map(sorted).collect()),
        Value::Number(n) => Value::Number(as_double(&n)),
        other => other,
    }
}

fn as_double(n: &Number) -> Number {
    match n.as_f64() {
        Some(f) if f.fract() == 0.0 && f.abs() < 1e15 => Number::from(f as i64),
        Some(f) => Number::from_f64(f).unwrap_or_else(|| n.clone()),
        None => n.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_nested_properties() {
        let out =
            sort_by_property_name(r#"{"b":1,"a":{"z":[{"y":1.5,"x":2.0}],"c":null}}"#).unwrap();
        let a = out.find("\"a\"").unwrap();
        let b = out.find("\"b\"").unwrap();
        assert!(a < b);
        assert!(
            out.find("\"x\": 2").unwrap() < out.find("\"y\": 1.5").unwrap()
                || out.contains("\"x\": 2,")
        );
        assert!(sort_by_property_name("{").is_err());
    }

    #[test]
    fn names_sort_by_culture() {
        let out = sort_by_property_name(r#"{"Zed":1,"apple":2,"Apple":3}"#).unwrap();
        let at = |n: &str| out.find(n).unwrap();
        assert!(at("\"apple\"") < at("\"Apple\"") && at("\"Apple\"") < at("\"Zed\""));
    }
}
