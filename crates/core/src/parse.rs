//! Title parsing helpers — quality, language, normalization.

use std::sync::OnceLock;

use regex::Regex;

use crate::entry::{Language, Quality};

/// Normalize a title for stable hashing / search:
/// lowercase, collapse whitespace, drop punctuation noise.
pub fn normalize_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut last_was_space = true;
    for c in title.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_alphanumeric() {
            out.push(c);
            last_was_space = false;
        } else if (c.is_whitespace() || matches!(c, '.' | '_' | '-' | '[' | ']' | '(' | ')'))
            && !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        // Other punctuation is dropped entirely.
    }
    out.trim().to_string()
}

static QUALITY_RE: OnceLock<Regex> = OnceLock::new();

/// Try to extract a [`Quality`] from a title or tag list.
/// Returns `None` if no recognizable marker is found.
pub fn parse_quality(title: &str, tags: &[String]) -> Option<Quality> {
    let re = QUALITY_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(2160p|4k|uhd|1080p|fhd|720p|hd|480p|sd|dvdrip|brrip|webrip|webdl|web-dl|bluray|blu-ray)\b").unwrap()
    });

    let from = |s: &str| -> Option<Quality> {
        let m = re.find(s)?.as_str().to_ascii_lowercase();
        Some(match m.as_str() {
            "2160p" | "4k" | "uhd" => Quality::P2160,
            "1080p" | "fhd" => Quality::P1080,
            "720p" | "hd" => Quality::P720,
            "480p" | "sd" => Quality::P480,
            other => Quality::Other(other.to_string()),
        })
    };

    if let Some(q) = from(title) {
        return Some(q);
    }
    for t in tags {
        if let Some(q) = from(t) {
            return Some(q);
        }
    }
    None
}

static LANG_RE: OnceLock<Regex> = OnceLock::new();

/// Extract languages from a title or tag list.
/// Returns the unique set of recognizable language markers found.
pub fn parse_languages(title: &str, tags: &[String]) -> Vec<Language> {
    let re = LANG_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(vostfr|vost|multi|truefrench|french|vff|vfq|vf|english|en)\b").unwrap()
    });

    let mut found: Vec<Language> = Vec::new();
    let push = |found: &mut Vec<Language>, lang: Language| {
        if !found.contains(&lang) {
            found.push(lang);
        }
    };

    let scan = |s: &str, found: &mut Vec<Language>| {
        for m in re.find_iter(s) {
            match m.as_str().to_ascii_lowercase().as_str() {
                "vostfr" | "vost" => push(found, Language::VOSTFR),
                "multi" => push(found, Language::Multi),
                "truefrench" | "french" | "vff" | "vfq" | "vf" => push(found, Language::FR),
                "english" | "en" => push(found, Language::EN),
                _ => {}
            }
        }
    };

    scan(title, &mut found);
    for t in tags {
        scan(t, &mut found);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_punctuation_and_lowers() {
        assert_eq!(
            normalize_title("Naruto.Shippuden [VOSTFR] (1080p)"),
            "naruto shippuden vostfr 1080p"
        );
    }

    #[test]
    fn normalize_collapses_whitespace() {
        assert_eq!(normalize_title("  hello   world  "), "hello world");
    }

    #[test]
    fn normalize_is_deterministic() {
        let a = normalize_title("Inception (2010) [4K] BluRay");
        let b = normalize_title("inception 2010 4k bluray");
        assert_eq!(a, b);
    }

    #[test]
    fn parses_quality_from_title() {
        assert_eq!(
            parse_quality("Inception 1080p BluRay", &[]),
            Some(Quality::P1080)
        );
        assert_eq!(parse_quality("Movie.4K.HDR", &[]), Some(Quality::P2160));
        assert_eq!(parse_quality("Untitled", &[]), None);
    }

    #[test]
    fn parses_quality_from_tags() {
        assert_eq!(
            parse_quality("Untitled", &["1080p".into()]),
            Some(Quality::P1080)
        );
    }

    #[test]
    fn parses_languages_unique() {
        let langs = parse_languages("Naruto VOSTFR Multi", &[]);
        assert!(langs.contains(&Language::VOSTFR));
        assert!(langs.contains(&Language::Multi));
    }

    #[test]
    fn parses_french_variants() {
        assert!(parse_languages("Movie TRUEFRENCH 1080p", &[]).contains(&Language::FR));
        assert!(parse_languages("Movie VF 1080p", &[]).contains(&Language::FR));
    }
}
