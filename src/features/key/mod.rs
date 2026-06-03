//! Key detection modules
//!
//! Detect musical key using:
//! - Krumhansl-Kessler templates (24 keys)
//! - Template matching
//! - Key clarity scoring

pub mod detector;
pub mod key_changes;
pub mod key_clarity;
pub mod templates;

pub use detector::{
    detect_key, detect_key_ensemble, detect_key_multi_scale, detect_key_weighted,
    detect_key_weighted_mode_heuristic,
};
pub use key_changes::{detect_key_changes, KeyChange, KeyChangeResult};
pub use key_clarity::compute_key_clarity;
pub use templates::KeyTemplates;

use crate::analysis::result::Key;
use std::cmp::Ordering;

pub(crate) fn key_sort_index(key: Key) -> u32 {
    match key {
        Key::Major(i) => i % 12,
        Key::Minor(i) => 12 + (i % 12),
    }
}

pub(crate) fn compare_key_scores_desc(a: &(Key, f32), b: &(Key, f32)) -> Ordering {
    match (a.1.is_finite(), b.1.is_finite()) {
        (true, true) => b.1.total_cmp(&a.1),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => Ordering::Equal,
    }
    .then_with(|| key_sort_index(a.0).cmp(&key_sort_index(b.0)))
}

pub(crate) fn sort_key_scores_desc(scores: &mut [(Key, f32)]) {
    scores.sort_by(compare_key_scores_desc);
}

/// Key detection result
#[derive(Debug, Clone)]
pub struct KeyDetectionResult {
    /// Detected key (best match)
    pub key: Key,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// All 24 key scores (ranked, highest first)
    pub all_scores: Vec<(Key, f32)>,

    /// Top N keys with scores (default: top 3)
    /// Useful for ambiguous cases or DJ key mixing
    pub top_keys: Vec<(Key, f32)>,
}
