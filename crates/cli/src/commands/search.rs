use anyhow::{Result, bail};
use clap::Args;
use tt_core::{Category, Language, Quality};
use tt_storage::{Database, SearchQuery, SearchScope};

use crate::fmt::{humanize_bytes, relative_time, short_id};

#[derive(Args)]
pub struct SearchArgs {
    /// Free-text query (matched against entry titles via FTS5).
    pub text: Vec<String>,

    /// Restrict to a specific source (id prefix) or pool (id prefix). Format:
    /// `source:<prefix>` or `pool:<prefix>`. Default: search all entries.
    #[arg(long)]
    pub r#in: Option<String>,

    /// Filter by category. Repeat for multiple values.
    #[arg(long, value_delimiter = ',')]
    pub category: Vec<String>,

    /// Filter by quality. Repeat for multiple values. (480p, 720p, 1080p, 4k)
    #[arg(long, value_delimiter = ',')]
    pub quality: Vec<String>,

    /// Filter by language. Repeat for multiple values. (vf, vostfr, en, multi)
    #[arg(long, value_delimiter = ',')]
    pub language: Vec<String>,

    /// Minimum size (bytes).
    #[arg(long)]
    pub min_size: Option<u64>,

    /// Maximum size (bytes).
    #[arg(long)]
    pub max_size: Option<u64>,

    /// Minimum number of seeders.
    #[arg(long)]
    pub min_seeders: Option<u32>,

    /// Cap the number of results.
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

pub fn run(args: SearchArgs, db: &Database) -> Result<()> {
    let scope = parse_scope(args.r#in.as_deref(), db)?;

    let categories: Option<Vec<Category>> = if args.category.is_empty() {
        None
    } else {
        Some(
            args.category
                .iter()
                .map(|s| parse_category(s))
                .collect::<Result<Vec<_>>>()?,
        )
    };

    let qualities: Option<Vec<Quality>> = if args.quality.is_empty() {
        None
    } else {
        Some(
            args.quality
                .iter()
                .map(|s| parse_quality(s))
                .collect::<Result<Vec<_>>>()?,
        )
    };

    let languages: Option<Vec<Language>> = if args.language.is_empty() {
        None
    } else {
        Some(
            args.language
                .iter()
                .map(|s| parse_language(s))
                .collect::<Result<Vec<_>>>()?,
        )
    };

    let q = SearchQuery {
        text: if args.text.is_empty() {
            None
        } else {
            Some(args.text.join(" "))
        },
        scope,
        categories,
        qualities,
        languages,
        size_min: args.min_size,
        size_max: args.max_size,
        seeders_min: args.min_seeders,
        limit: Some(args.limit),
    };

    let hits = db.search(&q)?;

    if hits.is_empty() {
        println!("no results.");
        return Ok(());
    }

    let sources = db.list_sources()?;
    println!("{} result(s):", hits.len());
    for hit in hits {
        let prov = hit
            .provenance
            .iter()
            .map(|sid| {
                sources
                    .iter()
                    .find(|s| s.id == *sid)
                    .map(|s| s.display_name.clone())
                    .unwrap_or_else(|| short_id(&sid.0.to_string()))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let size = hit
            .entry
            .size_bytes
            .map(humanize_bytes)
            .unwrap_or_else(|| "?".into());
        let qual = hit.entry.quality.as_ref().map(quality_label).unwrap_or("");
        let lang = hit
            .entry
            .languages
            .iter()
            .map(language_label)
            .collect::<Vec<_>>()
            .join("/");
        let seeders = hit
            .entry
            .seeders
            .map(|n| format!("{n} seeders"))
            .unwrap_or_default();
        println!(
            "  {}  {}",
            short_id(&hit.entry.id.as_hex()),
            hit.entry.title
        );
        println!(
            "       {} · {} · {} · {} · added {}  [{}]",
            size,
            qual,
            lang,
            seeders,
            relative_time(hit.entry.added_at),
            prov,
        );
    }
    Ok(())
}

fn parse_scope(s: Option<&str>, db: &Database) -> Result<SearchScope> {
    let Some(s) = s else {
        return Ok(SearchScope::All);
    };
    let (kind, prefix) = s.split_once(':').unwrap_or(("source", s));
    match kind {
        "source" | "src" => {
            let sources = db.list_sources()?;
            let m: Vec<_> = sources
                .iter()
                .filter(|s| s.id.0.to_string().starts_with(prefix))
                .collect();
            match m.len() {
                0 => bail!("no source matches '{prefix}'"),
                1 => Ok(SearchScope::Source(m[0].id)),
                n => bail!("{n} sources match '{prefix}'"),
            }
        }
        "pool" => {
            let pools = db.list_pools()?;
            let m: Vec<_> = pools
                .iter()
                .filter(|p| p.id.0.to_string().starts_with(prefix))
                .collect();
            match m.len() {
                0 => bail!("no pool matches '{prefix}'"),
                1 => Ok(SearchScope::Pool(m[0].id)),
                n => bail!("{n} pools match '{prefix}'"),
            }
        }
        other => bail!("unknown scope kind '{other}'. Use `source:<id>` or `pool:<id>`."),
    }
}

fn parse_category(s: &str) -> Result<Category> {
    Ok(match s.to_ascii_lowercase().as_str() {
        "films" | "movies" | "film" | "movie" => Category::Films,
        "series" | "serie" | "show" | "shows" | "tv" => Category::Series,
        "games" | "game" | "jeu" | "jeux" => Category::Games,
        "music" | "musique" => Category::Music,
        "books" | "book" | "livre" | "livres" => Category::Books,
        "software" | "soft" | "logiciel" | "logiciels" => Category::Software,
        "other" | "autre" => Category::Other,
        other => bail!("unknown category '{other}'"),
    })
}

fn parse_quality(s: &str) -> Result<Quality> {
    Ok(match s.to_ascii_lowercase().as_str() {
        "480p" => Quality::P480,
        "720p" | "hd" => Quality::P720,
        "1080p" | "fhd" => Quality::P1080,
        "2160p" | "4k" | "uhd" => Quality::P2160,
        other => Quality::Other(other.to_string()),
    })
}

fn parse_language(s: &str) -> Result<Language> {
    Ok(match s.to_ascii_lowercase().as_str() {
        "fr" | "vf" | "french" => Language::FR,
        "vostfr" | "vost" => Language::VOSTFR,
        "en" | "english" => Language::EN,
        "multi" => Language::Multi,
        other => Language::Other(other.to_string()),
    })
}

fn quality_label(q: &Quality) -> &str {
    match q {
        Quality::P480 => "480p",
        Quality::P720 => "720p",
        Quality::P1080 => "1080p",
        Quality::P2160 => "4K",
        Quality::Other(s) => s.as_str(),
    }
}

fn language_label(l: &Language) -> &str {
    match l {
        Language::FR => "VF",
        Language::VOSTFR => "VOSTFR",
        Language::EN => "EN",
        Language::Multi => "Multi",
        Language::Other(s) => s.as_str(),
    }
}
