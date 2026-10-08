use core_common::{CoreError, CoreResult};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Document {
    id: u64,
    text: String,
    normalized: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub text: String,
}

#[derive(Default)]
pub struct SearchEngine {
    documents: HashMap<u64, Document>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: u64, text: &str) -> CoreResult<()> {
        if text.is_empty() {
            return Err(CoreError::InvalidArgument("text must not be empty".into()));
        }
        self.documents.insert(
            id,
            Document {
                id,
                text: text.to_owned(),
                normalized: normalize(text),
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> bool {
        self.documents.remove(&id).is_some()
    }

    pub fn clear(&mut self) {
        self.documents.clear();
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        if query.is_empty() || limit == 0 {
            return Vec::new();
        }
        let query = normalize(query);
        let mut results = self
            .documents
            .values()
            .filter_map(|doc| {
                let score = score(&doc.normalized, &query);
                (score > 0.0).then(|| SearchResult {
                    id: doc.id,
                    score,
                    text: doc.text.clone(),
                })
            })
            .collect::<Vec<_>>();

        results.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(limit);
        results
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn score(text: &str, query: &str) -> f32 {
    if text == query { return 100.0; }
    if text.starts_with(query) { return 80.0 + query.len() as f32 / text.len().max(1) as f32; }
    if text.contains(query) { return 60.0 + query.len() as f32 / text.len().max(1) as f32; }

    let distance = levenshtein(text, query);
    let max_len = text.chars().count().max(query.chars().count());
    if distance <= 2 && max_len > 0 {
        return 20.0 + (max_len - distance) as f32 / max_len as f32;
    }
    0.0
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    for (i, a_char) in a.chars().enumerate() {
        let mut current = vec![i + 1; b_chars.len() + 1];
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = usize::from(a_char != *b_char);
            current[j + 1] = (current[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        prev = current;
    }
    prev[b_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_works() {
        let mut engine = SearchEngine::new();
        engine.add(1, "床前明月光").unwrap();
        engine.add(2, "明月几时有").unwrap();
        let results = engine.search("明月", 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 2);
    }

    #[test]
    fn remove_works() {
        let mut engine = SearchEngine::new();
        engine.add(1, "hello").unwrap();
        assert!(engine.remove(1));
        assert_eq!(engine.len(), 0);
    }
}
