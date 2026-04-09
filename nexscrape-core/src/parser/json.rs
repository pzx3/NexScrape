//! JSON data extractor using JSONPath-like expressions.

use crate::{NexError, Result};
use serde_json::Value;

/// JSON data extractor.
///
/// Supports simple dot-notation paths for extracting values from JSON.
pub struct JsonExtractor {
    data: Value,
}

impl JsonExtractor {
    /// Create a new JSON extractor from a JSON string.
    pub fn from_str(json: &str) -> Result<Self> {
        let data: Value = serde_json::from_str(json)
            .map_err(|e| NexError::ParseError(format!("JSON parse error: {}", e)))?;
        Ok(Self { data })
    }

    /// Create a new JSON extractor from a serde_json Value.
    pub fn from_value(data: Value) -> Self {
        Self { data }
    }

    /// Extract a value at the given dot-notation path.
    ///
    /// # Example
    /// ```
    /// use nexscrape_core::parser::json::JsonExtractor;
    ///
    /// let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
    /// let extractor = JsonExtractor::from_str(json).unwrap();
    /// let name = extractor.get("user.name").unwrap();
    /// assert_eq!(name, &serde_json::Value::String("Alice".to_string()));
    /// ```
    pub fn get(&self, path: &str) -> Option<&Value> {
        let mut current = &self.data;

        for key in path.split('.') {
            // Try array index
            if let Ok(index) = key.parse::<usize>() {
                current = current.get(index)?;
            } else {
                current = current.get(key)?;
            }
        }

        Some(current)
    }

    /// Extract a string value at the given path.
    pub fn get_str(&self, path: &str) -> Option<&str> {
        self.get(path)?.as_str()
    }

    /// Extract an integer value at the given path.
    pub fn get_i64(&self, path: &str) -> Option<i64> {
        self.get(path)?.as_i64()
    }

    /// Extract a float value at the given path.
    pub fn get_f64(&self, path: &str) -> Option<f64> {
        self.get(path)?.as_f64()
    }

    /// Extract a boolean value at the given path.
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path)?.as_bool()
    }

    /// Extract an array at the given path and apply an extractor to each element.
    pub fn get_array(&self, path: &str) -> Option<&Vec<Value>> {
        self.get(path)?.as_array()
    }

    /// Extract multiple fields into a HashMap.
    pub fn extract_fields(
        &self,
        fields: &std::collections::HashMap<String, String>,
    ) -> std::collections::HashMap<String, Value> {
        let mut result = std::collections::HashMap::new();

        for (name, path) in fields {
            if let Some(value) = self.get(path) {
                result.insert(name.clone(), value.clone());
            }
        }

        result
    }

    /// Get the root JSON value.
    pub fn root(&self) -> &Value {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JSON: &str = r#"{
        "store": {
            "name": "BookStore",
            "books": [
                {"title": "Rust Programming", "price": 39.99, "in_stock": true},
                {"title": "Web Scraping 101", "price": 29.99, "in_stock": false}
            ],
            "location": {
                "city": "Berlin",
                "country": "Germany"
            }
        }
    }"#;

    #[test]
    fn test_get_string() {
        let ext = JsonExtractor::from_str(TEST_JSON).unwrap();
        assert_eq!(ext.get_str("store.name").unwrap(), "BookStore");
    }

    #[test]
    fn test_get_nested() {
        let ext = JsonExtractor::from_str(TEST_JSON).unwrap();
        assert_eq!(ext.get_str("store.location.city").unwrap(), "Berlin");
    }

    #[test]
    fn test_get_array_index() {
        let ext = JsonExtractor::from_str(TEST_JSON).unwrap();
        assert_eq!(
            ext.get_str("store.books.0.title").unwrap(),
            "Rust Programming"
        );
    }

    #[test]
    fn test_get_number() {
        let ext = JsonExtractor::from_str(TEST_JSON).unwrap();
        assert_eq!(ext.get_f64("store.books.0.price").unwrap(), 39.99);
    }

    #[test]
    fn test_get_bool() {
        let ext = JsonExtractor::from_str(TEST_JSON).unwrap();
        assert_eq!(ext.get_bool("store.books.0.in_stock").unwrap(), true);
        assert_eq!(ext.get_bool("store.books.1.in_stock").unwrap(), false);
    }

    #[test]
    fn test_missing_path() {
        let ext = JsonExtractor::from_str(TEST_JSON).unwrap();
        assert!(ext.get("store.nonexistent").is_none());
    }
}
