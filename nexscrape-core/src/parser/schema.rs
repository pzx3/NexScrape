//! Schema-based data validation and extraction.

use crate::{NexError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Supported field types for schema validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Url,
    Email,
    List,
    Any,
}

/// Schema field definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub selector: Option<String>,
    pub default: Option<Value>,
    pub transform: Option<String>,
}

impl SchemaField {
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            required: false,
            selector: None,
            default: None,
            transform: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }
}

/// Schema definition for validating extracted data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: Vec<SchemaField>,
}

impl Schema {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, field: SchemaField) -> Self {
        self.fields.push(field);
        self
    }

    /// Validate a data item against this schema.
    pub fn validate(
        &self,
        data: &std::collections::HashMap<String, Value>,
    ) -> Result<()> {
        for field in &self.fields {
            match data.get(&field.name) {
                Some(value) => {
                    self.validate_type(&field.name, value, &field.field_type)?;
                }
                None => {
                    if field.required {
                        return Err(NexError::SchemaError(format!(
                            "Required field '{}' is missing",
                            field.name
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_type(&self, name: &str, value: &Value, field_type: &FieldType) -> Result<()> {
        let valid = match field_type {
            FieldType::String => value.is_string(),
            FieldType::Integer => value.is_i64() || value.is_u64(),
            FieldType::Float => value.is_f64() || value.is_i64(),
            FieldType::Boolean => value.is_boolean(),
            FieldType::Url => {
                value.as_str().map_or(false, |s| {
                    url::Url::parse(s).is_ok()
                })
            }
            FieldType::Email => {
                value.as_str().map_or(false, |s| {
                    s.contains('@') && s.contains('.')
                })
            }
            FieldType::List => value.is_array(),
            FieldType::Any => true,
        };

        if !valid {
            return Err(NexError::SchemaError(format!(
                "Field '{}' expected type {:?}, got: {}",
                name, field_type, value
            )));
        }

        Ok(())
    }

    /// Apply default values to missing fields.
    pub fn apply_defaults(
        &self,
        data: &mut std::collections::HashMap<String, Value>,
    ) {
        for field in &self.fields {
            if !data.contains_key(&field.name) {
                if let Some(ref default) = field.default {
                    data.insert(field.name.clone(), default.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validation_pass() {
        let schema = Schema::new("product")
            .field(SchemaField::new("name", FieldType::String).required())
            .field(SchemaField::new("price", FieldType::Float).required())
            .field(SchemaField::new("in_stock", FieldType::Boolean));

        let mut data = std::collections::HashMap::new();
        data.insert("name".to_string(), json!("Widget"));
        data.insert("price".to_string(), json!(9.99));
        data.insert("in_stock".to_string(), json!(true));

        assert!(schema.validate(&data).is_ok());
    }

    #[test]
    fn test_schema_validation_missing_required() {
        let schema = Schema::new("product")
            .field(SchemaField::new("name", FieldType::String).required());

        let data = std::collections::HashMap::new();
        assert!(schema.validate(&data).is_err());
    }

    #[test]
    fn test_schema_validation_wrong_type() {
        let schema = Schema::new("product")
            .field(SchemaField::new("price", FieldType::Float).required());

        let mut data = std::collections::HashMap::new();
        data.insert("price".to_string(), json!("not a number"));

        assert!(schema.validate(&data).is_err());
    }

    #[test]
    fn test_schema_defaults() {
        let schema = Schema::new("product")
            .field(
                SchemaField::new("status", FieldType::String)
                    .default_value(json!("unknown")),
            );

        let mut data = std::collections::HashMap::new();
        schema.apply_defaults(&mut data);

        assert_eq!(data.get("status").unwrap(), &json!("unknown"));
    }
}
