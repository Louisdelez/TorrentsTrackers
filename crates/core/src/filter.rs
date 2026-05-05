//! Filter logic — apply [`PoolFilters`] to entries.

use crate::entry::Entry;
use crate::pool::PoolFilters;

impl PoolFilters {
    /// Return `true` if the entry passes all configured filters.
    pub fn matches(&self, entry: &Entry) -> bool {
        // Categories
        if let Some(cats) = &self.categories
            && !cats.contains(&entry.category)
        {
            return false;
        }

        // Required tags (all must be present)
        for required in &self.tags_required {
            if !entry.tags.iter().any(|t| t.eq_ignore_ascii_case(required)) {
                return false;
            }
        }

        // Excluded tags (none must be present)
        for excluded in &self.tags_excluded {
            if entry.tags.iter().any(|t| t.eq_ignore_ascii_case(excluded)) {
                return false;
            }
        }

        // Qualities
        if let Some(qs) = &self.qualities {
            match &entry.quality {
                Some(q) if qs.contains(q) => {}
                _ => return false,
            }
        }

        // Languages — at least one match required if filter is set
        if let Some(ls) = &self.languages
            && !entry.languages.iter().any(|l| ls.contains(l))
        {
            return false;
        }

        // Size bounds
        if let Some(min) = self.size_min
            && entry.size_bytes.is_some_and(|s| s < min)
        {
            return false;
        }
        if let Some(max) = self.size_max
            && entry.size_bytes.is_some_and(|s| s > max)
        {
            return false;
        }

        // Seeders min
        if let Some(min) = self.seeders_min
            && entry.seeders.is_some_and(|s| s < min)
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::entry::{
        Category, ContentLink, Entry, Language, PublicKeyBytes, Quality, SignatureBytes,
    };
    use crate::ids::{ContentId, SourceId};

    fn fixture() -> Entry {
        Entry {
            id: ContentId([0; 32]),
            title: "Inception 1080p".to_string(),
            link: ContentLink::Magnet(String::new()),
            category: Category::Films,
            tags: vec!["1080p".into(), "vostfr".into()],
            quality: Some(Quality::P1080),
            languages: vec![Language::VOSTFR],
            size_bytes: Some(5 * 1024 * 1024 * 1024), // 5 GB
            seeders: Some(120),
            leechers: Some(4),
            added_at: Utc::now(),
            contributor_pubkey: PublicKeyBytes([0; 32]),
            source_id: SourceId::new(),
            signature: SignatureBytes([0; 64]),
            description: None,
            poster_url: None,
        }
    }

    #[test]
    fn matches_when_all_pass() {
        let f = PoolFilters {
            categories: Some(vec![Category::Films]),
            qualities: Some(vec![Quality::P1080]),
            ..Default::default()
        };
        assert!(f.matches(&fixture()));
    }

    #[test]
    fn rejects_wrong_category() {
        let f = PoolFilters {
            categories: Some(vec![Category::Series]),
            ..Default::default()
        };
        assert!(!f.matches(&fixture()));
    }

    #[test]
    fn requires_all_tags() {
        let f = PoolFilters {
            tags_required: vec!["1080p".into(), "missing".into()],
            ..Default::default()
        };
        assert!(!f.matches(&fixture()));
    }

    #[test]
    fn excludes_tags() {
        let f = PoolFilters {
            tags_excluded: vec!["VOSTFR".into()],
            ..Default::default()
        };
        assert!(!f.matches(&fixture()));
    }

    #[test]
    fn enforces_size_bounds() {
        let f = PoolFilters {
            size_max: Some(1024 * 1024 * 1024),
            ..Default::default()
        };
        assert!(!f.matches(&fixture()));
    }

    #[test]
    fn enforces_seeders_min() {
        let f = PoolFilters {
            seeders_min: Some(200),
            ..Default::default()
        };
        assert!(!f.matches(&fixture()));
    }
}
