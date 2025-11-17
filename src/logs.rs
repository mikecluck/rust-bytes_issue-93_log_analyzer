use std::{collections::HashMap, u32};

use serde::Serialize;
use stopwords::Stopwords;

#[allow(dead_code)]
pub(crate) struct LogEntry {
    pub timestamp: String,
    pub hostname: String,
    pub process_name: String,
    pub pid: u32,
    pub message: String,
}

pub(crate) struct LogStatsBuilder {
    total_entries: usize,
    by_process: HashMap<String, u32>,
    by_hostname: HashMap<String, u32>,
    most_frequent_process: String,
    most_frequent_hostname: String,
    top_keywords: Vec<String>,
    keyword_count: HashMap<String, u32>,
}

#[derive(Serialize)]
pub(crate) struct LogStats {
    pub total_entries: usize,
    pub by_process: HashMap<String, u32>,
    pub by_hostname: HashMap<String, u32>,
    pub most_frequent_process: String,
    pub most_frequent_hostname: String,
    pub top_keywords: Vec<String>,
}

impl LogStatsBuilder {
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            by_process: HashMap::new(),
            by_hostname: HashMap::new(),
            most_frequent_process: String::new(),
            most_frequent_hostname: String::new(),
            top_keywords: Vec::new(),
            keyword_count: HashMap::new(),
        }
    }

    pub fn add_log_entry(&mut self, log_entry: LogEntry) {
        self.total_entries += 1;

        let by_process_count = self.by_process.get(&log_entry.process_name).unwrap_or(&0);
        self.by_process
            .insert(log_entry.process_name, *by_process_count + 1);

        let by_hostname_count = self.by_hostname.get(&log_entry.hostname).unwrap_or(&0);
        self.by_hostname
            .insert(log_entry.hostname, *by_hostname_count + 1);

        // Keep track of how many times each keyword appears
        let stopwords = stopwords::NLTK::stopwords(stopwords::Language::English).unwrap();
        let keywords = log_entry
            .message
            .split(|c: char| !c.is_alphabetic())
            .map(|s| s.to_lowercase())
            .filter(|s| !stopwords.contains(&s.as_str()));

        for keyword in keywords {
            let keyword = keyword.trim();
            if keyword.is_empty() {
                continue;
            }

            let count = self.keyword_count.get(keyword).unwrap_or(&0);
            self.keyword_count.insert(keyword.into(), count + 1);
        }
    }

    /// Calculate stats about the whole collection
    pub fn build(mut self) -> LogStats {
        let mut most_frequent_process_name = "";
        let mut most_frequent_process_count: u32 = 0;
        for (process, count) in &self.by_process {
            if count > &most_frequent_process_count {
                most_frequent_process_count = *count;
                most_frequent_process_name = process;
            }
        }

        self.most_frequent_process = most_frequent_process_name.to_string();

        let mut most_frequent_hostname_name = "";
        let mut most_frequent_hostname_count: u32 = 0;
        for (hostname, count) in &self.by_hostname {
            if count > &most_frequent_hostname_count {
                most_frequent_hostname_count = *count;
                most_frequent_hostname_name = hostname;
            }
        }

        self.most_frequent_hostname = most_frequent_hostname_name.to_string();

        let mut keywords_by_count = self.keyword_count.iter().collect::<Vec<_>>();
        keywords_by_count.sort_by(|a, b| a.1.cmp(b.1).reverse().then(a.0.cmp(b.0)));

        let mut top_keywords = Vec::new();

        let mut prev_count = &u32::MAX;
        for (key, count) in keywords_by_count {
            if top_keywords.len() > 10 && count != prev_count {
                break;
            }

            prev_count = count;
            top_keywords.push(key.clone());
        }

        self.top_keywords = top_keywords;

        LogStats {
            total_entries: self.total_entries,
            by_process: self.by_process,
            by_hostname: self.by_hostname,
            most_frequent_process: self.most_frequent_process,
            most_frequent_hostname: self.most_frequent_hostname,
            top_keywords: self.top_keywords,
        }
    }
}
