//! Validate project property overrides before starting a candidate.
use anyhow::{ensure, Context, Result};
use serde_json::Value;

pub fn merge(schema: &Value, updates: &Value) -> Result<Value> {
    ensure!(
        serde_json::to_vec(updates)?.len() <= 65536,
        "Property updates exceed 64 KiB"
    );
    let updates = updates
        .as_object()
        .context("Properties must be an object")?;
    for (name, update) in updates {
        let declared = schema.get(name).context("Undeclared wallpaper property")?;
        let value = update.get("value").context("Property value missing")?;
        let original = declared
            .get("value")
            .context("Declared property value missing")?;
        ensure!(
            (original.is_boolean() && value.is_boolean())
                || (original.is_number() && value.is_number())
                || (original.is_string() && value.is_string()),
            "Wallpaper property value has incorrect type"
        );
        if declared.get("type").and_then(Value::as_str) == Some("color") {
            let components = value
                .as_str()
                .context("Color must be a string")?
                .split_whitespace()
                .map(str::parse::<f32>)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            ensure!(
                components.len() == 3
                    && components
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
                "Color requires three values in 0..1"
            );
        }
        if let Some(options) = declared.get("options").and_then(Value::as_array) {
            ensure!(
                options
                    .iter()
                    .any(
                        |option| option.get("value").is_some_and(|allowed| allowed == value
                            || (allowed.is_number()
                                && value.is_number()
                                && allowed.as_f64() == value.as_f64()))
                    ),
                "Property choice is not declared"
            );
        }
        if let Some(number) = value.as_f64() {
            ensure!(
                number.is_finite(),
                "Wallpaper property value must be finite"
            );
            if let Some(min) = declared.get("min").and_then(Value::as_f64) {
                ensure!(number >= min, "Property below minimum");
            }
            if let Some(max) = declared.get("max").and_then(Value::as_f64) {
                ensure!(number <= max, "Property above maximum");
            }
        }
        ensure!(
            update.as_object().is_some_and(|obj| obj.len() == 1),
            "Only property values may be updated"
        );
    }
    let mut merged = schema.clone();
    for (name, update) in updates {
        merged[name]["value"] = update["value"].clone();
    }
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn overrides_are_typed_bounded_and_transactional() {
        let schema = json!({
            "speed":{"type":"slider","value":0.5,"min":0,"max":1},
            "caption":{"type":"textinput","value":"original"},
            "enabled":{"type":"bool","value":true},
            "color":{"type":"color","value":"1 0 0"},
            "choice":{"type":"combo","value":1,"options":[{"value":1},{"value":2}]}
        });
        let updated = merge(&schema, &json!({"speed":{"value":0.75},"caption":{"value":"new"},"enabled":{"value":false},"color":{"value":"0 1 0"},"choice":{"value":2}})).unwrap();
        assert_eq!(updated["speed"]["value"], 0.75);
        assert_eq!(schema["caption"]["value"], "original");
        assert_eq!(updated["speed"]["min"], 0);
        for invalid in [
            json!(null),
            json!([]),
            json!({"unknown":{"value":1}}),
            json!({"speed":{"value":true}}),
            json!({"speed":{"value":2}}),
            json!({"speed":{"value":-1}}),
            json!({"caption":{"value":"x","type":"slider"}}),
            json!({"enabled":{"value":0}}),
            json!({"color":{"value":"NaN 0 0"}}),
            json!({"color":{"value":"2 0 0"}}),
            json!({"choice":{"value":3}}),
            json!({"caption":{"value":"x".repeat(65536)}}),
        ] {
            assert!(merge(&schema, &invalid).is_err(), "{invalid}");
        }
    }
}
