use std::collections::{HashMap, HashSet};

/// A small built-in stopword list. Enough for English; not exhaustive.
pub const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has",
    "have", "had", "he", "her", "his", "him", "i", "in", "is", "it", "its",
    "of", "on", "or", "she", "that", "the", "their", "there", "they", "this",
    "to", "was", "were", "we", "will", "with", "you", "your", "yours", "our",
    "us", "but", "not", "so", "if", "then", "than", "into", "about", "what",
    "when", "where", "which", "who", "why", "how", "do", "does", "did", "can",
    "could", "should", "would", "may", "might", "must", "shall", "no", "yes",
    "all", "any", "some", "one", "two", "three", "more", "less", "most",
    "least", "much", "many", "few", "just", "only", "also", "too", "very",
    "been", "being", "am", "my", "me", "yourself", "themselves", "ourselves",
    "himself", "herself", "itself", "myself",
];

/// Tokenize a body into lowercase words, with stopwords removed and short
/// tokens dropped. Returns a multiset (Vec<String>) of tokens.
pub fn tokenize(body: &str) -> Vec<String> {
    let stop: HashSet<&str> = STOPWORDS.iter().copied().collect();
    let mut out = Vec::new();
    let mut current = String::new();
    for c in body.chars() {
        if c.is_alphanumeric() || c == '\'' || c == '-' {
            for lc in c.to_lowercase() {
                current.push(lc);
            }
        } else if !current.is_empty() {
            if current.len() >= 3 && !stop.contains(current.as_str()) {
                out.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    if !current.is_empty() && current.len() >= 3 && !stop.contains(current.as_str()) {
        out.push(current);
    }
    out
}

/// Build a document-frequency map: for each token, how many documents contain it.
pub fn document_frequencies<'a>(
    docs: impl IntoIterator<Item = &'a [String]>,
) -> HashMap<String, usize> {
    let mut df: HashMap<String, usize> = HashMap::new();
    for tokens in docs {
        let uniq: HashSet<&String> = tokens.iter().collect();
        for t in uniq {
            *df.entry(t.clone()).or_insert(0) += 1;
        }
    }
    df
}

/// Inverse document frequency, smoothed. `n` is total documents.
pub fn idf(df: usize, n: usize) -> f32 {
    ((n as f32 + 1.0) / (df as f32 + 1.0)).ln() + 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        let t = tokenize("The garden is a place for thoughts and small dreams.");
        assert!(t.contains(&"garden".to_string()));
        assert!(t.contains(&"place".to_string()));
        assert!(t.contains(&"thoughts".to_string()));
        assert!(t.contains(&"dreams".to_string()));
        assert!(!t.contains(&"the".to_string()));
        assert!(!t.contains(&"is".to_string()));
    }

    #[test]
    fn idf_works() {
        let n = 10;
        let common = idf(9, n);
        let rare = idf(1, n);
        assert!(rare > common);
    }
}
