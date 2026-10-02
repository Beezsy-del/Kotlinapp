use crate::models::{AnalysisResult, HistoryRecord};
use chrono::Local;
use std::fs;
use std::path::PathBuf;

/// Persistent local history store for VerdictEdge contract scan records.
/// Saves and loads scans from local disk (~/.verdictedge/history.json).
#[derive(Debug, Clone)]
pub struct HistoryStore {
    storage_path: PathBuf,
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryStore {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let default_dir = home.join(".verdictedge");
        let default_path = default_dir.join("history.json");

        Self {
            storage_path: default_path,
        }
    }

    pub fn with_path(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    /// Load all historical scans sorted newest first
    pub fn load_all(&self) -> Vec<HistoryRecord> {
        if !self.storage_path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&self.storage_path) {
            Ok(json_str) => {
                let mut records: Vec<HistoryRecord> = serde_json::from_str(&json_str).unwrap_or_default();
                records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                records
            }
            Err(_) => Vec::new(),
        }
    }

    /// Save a new history record
    pub fn save_record(&self, record: HistoryRecord) -> Result<(), String> {
        let mut records = self.load_all();
        // Remove duplicate if same ID exists
        records.retain(|r| r.id != record.id);
        records.insert(0, record);

        // Keep maximum 50 most recent records to prevent unlimited file growth
        if records.len() > 50 {
            records.truncate(50);
        }

        self.persist_records(&records)
    }

    /// Delete a record by ID
    pub fn delete_record(&self, id: &str) -> Result<(), String> {
        let mut records = self.load_all();
        records.retain(|r| r.id != id);
        self.persist_records(&records)
    }

    /// Clear all scan history
    pub fn clear_all(&self) -> Result<(), String> {
        self.persist_records(&[])
    }

    /// Construct a HistoryRecord from an active contract text and its AnalysisResult
    pub fn record_from_analysis(contract_text: &str, result: &AnalysisResult) -> HistoryRecord {
        let now = Local::now();
        let timestamp = now.timestamp();
        let formatted_date = now.format("%Y-%m-%d %H:%M").to_string();
        let title = Self::extract_clean_title(contract_text);

        let clean_snippet = contract_text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("---"))
            .collect::<Vec<_>>()
            .join(" ");

        let snippet = if clean_snippet.len() > 140 {
            format!("{}...", &clean_snippet[..140])
        } else if !clean_snippet.is_empty() {
            clean_snippet
        } else {
            "Scanned Legal Agreement".to_string()
        };

        let id = format!("{}_{}", timestamp, title.replace(' ', "_").to_lowercase());

        HistoryRecord {
            id,
            timestamp,
            formatted_date,
            risk_level: result.risk_level,
            title,
            snippet,
            full_text: contract_text.to_string(),
        }
    }

    /// Extract a human-readable title from the top lines of the contract
    pub fn extract_clean_title(full_text: &str) -> String {
        for line in full_text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("---") || trimmed.is_empty() {
                continue;
            }
            if trimmed.len() >= 3 {
                let truncated: String = trimmed.chars().take(45).collect();
                return truncated.trim().to_string();
            }
        }
        "Legal Agreement".to_string()
    }

    fn persist_records(&self, records: &[HistoryRecord]) -> Result<(), String> {
        if let Some(parent) = self.storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(records)
            .map_err(|e| format!("Failed to serialize history records: {}", e))?;

        fs::write(&self.storage_path, json)
            .map_err(|e| format!("Failed to write history file {}: {}", self.storage_path.display(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RiskLevel;

    #[test]
    fn test_extract_clean_title() {
        let contract = "--- [Document 1: Master_Services_Agreement.pdf] ---\nMASTER CONSULTING SERVICES AGREEMENT\nBetween Party A and Party B";
        let title = HistoryStore::extract_clean_title(contract);
        assert!(!title.is_empty());
        assert!(title.contains("Document") || title.contains("MASTER CONSULTING"));
    }

    #[test]
    fn test_save_load_delete_history() {
        let temp_dir = std::env::temp_dir();
        let history_file = temp_dir.join("test_verdictedge_history.json");
        let store = HistoryStore::with_path(history_file.clone());

        let _ = store.clear_all();
        assert_eq!(store.load_all().len(), 0);

        let record = HistoryRecord {
            id: "rec_1".into(),
            timestamp: 1720000000,
            formatted_date: "2026-10-02 18:00".into(),
            risk_level: RiskLevel::High,
            title: "Test Employment Contract".into(),
            snippet: "Non-compete clause for 24 months".into(),
            full_text: "Full contract text here...".into(),
        };

        store.save_record(record.clone()).unwrap();
        let records = store.load_all();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].title, "Test Employment Contract");
        assert_eq!(records[0].risk_level, RiskLevel::High);

        store.delete_record("rec_1").unwrap();
        assert_eq!(store.load_all().len(), 0);

        let _ = fs::remove_file(history_file);
    }
}
