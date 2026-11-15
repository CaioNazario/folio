//! YAML values as JSON, for the `extra` metadata (spec 04).

use saphyr::{Scalar, Yaml};
use serde_json::{Number, Value};

/// `value` as JSON. Tags are dropped; what YAML cannot resolve becomes null.
pub fn to_json(value: &Yaml<'_>) -> Value {
    match value {
        Yaml::Value(scalar) => scalar_json(scalar),
        Yaml::Representation(text, ..) => Value::String(text.to_string()),
        Yaml::Sequence(items) => Value::Array(items.iter().map(to_json).collect()),
        Yaml::Mapping(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key_text(key), to_json(value)))
                .collect(),
        ),
        Yaml::Tagged(_, inner) => to_json(inner),
        Yaml::Alias(_) | Yaml::BadValue => Value::Null,
    }
}

fn scalar_json(scalar: &Scalar<'_>) -> Value {
    match scalar {
        Scalar::Null => Value::Null,
        Scalar::Boolean(flag) => Value::Bool(*flag),
        Scalar::Integer(number) => Value::from(*number),
        Scalar::FloatingPoint(number) => {
            Number::from_f64(number.into_inner()).map_or(Value::Null, Value::Number)
        }
        Scalar::String(text) => Value::String(text.to_string()),
    }
}

/// A mapping key as JSON object key: strings as they are, anything else in
/// its JSON form.
fn key_text(key: &Yaml<'_>) -> String {
    match key.as_str() {
        Some(text) => text.to_string(),
        None => to_json(key).to_string(),
    }
}
