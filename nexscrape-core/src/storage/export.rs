//! Data export to multiple formats (JSON, CSV, JSONL).

use crate::{Item, NexError, Result};
use serde::{Deserialize, Serialize};


/// Supported export formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    JsonLines,
    Csv,
}

/// Data exporter.
pub struct Exporter;

impl Exporter {
    /// Export items to a file.
    pub fn to_file(items: &[Item], path: &str, format: ExportFormat) -> Result<()> {
        let content = match format {
            ExportFormat::Json => Self::to_json(items)?,
            ExportFormat::JsonLines => Self::to_jsonl(items)?,
            ExportFormat::Csv => Self::to_csv(items)?,
        };

        std::fs::write(path, content)?;
        tracing::info!(path = %path, count = items.len(), "Exported items");
        Ok(())
    }

    /// Convert items to JSON string.
    pub fn to_json(items: &[Item]) -> Result<String> {
        serde_json::to_string_pretty(items)
            .map_err(|e| NexError::ExportError(format!("JSON serialization failed: {}", e)))
    }

    /// Convert items to JSON Lines (one JSON object per line).
    pub fn to_jsonl(items: &[Item]) -> Result<String> {
        let lines: Vec<String> = items
            .iter()
            .map(|item| serde_json::to_string(item))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| NexError::ExportError(format!("JSONL serialization failed: {}", e)))?;
        Ok(lines.join("\n"))
    }

    /// Convert items to CSV string.
    pub fn to_csv(items: &[Item]) -> Result<String> {
        if items.is_empty() {
            return Ok(String::new());
        }

        // Collect all field names
        let mut all_fields: Vec<String> = Vec::new();
        for item in items {
            for key in item.fields.keys() {
                if !all_fields.contains(key) {
                    all_fields.push(key.clone());
                }
            }
        }
        all_fields.sort();

        let mut output = Vec::new();

        // Write CSV using the csv crate
        {
            let mut wtr = csv::Writer::from_writer(&mut output);

            // Header
            let mut header = all_fields.clone();
            header.push("source_url".to_string());
            wtr.write_record(&header)
                .map_err(|e| NexError::ExportError(format!("CSV header failed: {}", e)))?;

            // Rows
            for item in items {
                let mut row: Vec<String> = all_fields
                    .iter()
                    .map(|field| {
                        item.fields
                            .get(field)
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .unwrap_or_default()
                    })
                    .collect();
                row.push(item.source_url.clone());

                wtr.write_record(&row)
                    .map_err(|e| NexError::ExportError(format!("CSV row failed: {}", e)))?;
            }

            wtr.flush()
                .map_err(|e| NexError::ExportError(format!("CSV flush failed: {}", e)))?;
        }

        String::from_utf8(output)
            .map_err(|e| NexError::ExportError(format!("CSV UTF-8 error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_items() -> Vec<Item> {
        vec![
            Item::new("https://example.com/1")
                .set("title", json!("First Product"))
                .set("price", json!(9.99)),
            Item::new("https://example.com/2")
                .set("title", json!("Second Product"))
                .set("price", json!(19.99)),
        ]
    }

    #[test]
    fn test_export_json() {
        let items = test_items();
        let json = Exporter::to_json(&items).unwrap();
        assert!(json.contains("First Product"));
        assert!(json.contains("19.99"));
    }

    #[test]
    fn test_export_jsonl() {
        let items = test_items();
        let jsonl = Exporter::to_jsonl(&items).unwrap();
        let lines: Vec<&str> = jsonl.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_export_csv() {
        let items = test_items();
        let csv = Exporter::to_csv(&items).unwrap();
        assert!(csv.contains("title"));
        assert!(csv.contains("price"));
        assert!(csv.contains("First Product"));
    }

    #[test]
    fn test_export_empty() {
        let csv = Exporter::to_csv(&[]).unwrap();
        assert!(csv.is_empty());
    }
}
