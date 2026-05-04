use anyhow::{Result, bail};
use clap::Args;
use tt_core::ContentLink;
use tt_storage::{Database, SearchQuery};

#[derive(Args)]
pub struct OpenArgs {
    /// Entry id (prefix of the ContentId hex). Use `tt search` to find one.
    pub id: String,
    /// Print the magnet instead of launching a torrent client.
    #[arg(long)]
    pub print: bool,
}

pub fn run(args: OpenArgs, db: &Database) -> Result<()> {
    // Pull a generous slice and prefix-match locally — Phase 1 simplification.
    let q = SearchQuery {
        limit: Some(5_000),
        ..Default::default()
    };
    let hits = db.search(&q)?;
    let matches: Vec<_> = hits
        .into_iter()
        .filter(|h| h.entry.id.as_hex().starts_with(&args.id))
        .collect();
    let entry = match matches.len() {
        0 => bail!("no entry matches id prefix '{}'", args.id),
        1 => matches.into_iter().next().unwrap().entry,
        n => bail!(
            "{n} entries match prefix '{}'; provide more characters",
            args.id
        ),
    };

    let magnet = match &entry.link {
        ContentLink::Magnet(s) => s.clone(),
        ContentLink::TorrentUrl(u) => u.clone(),
        ContentLink::InfoHash(h) => tt_core::magnet::build_magnet(h, Some(&entry.title)),
    };

    if args.print {
        println!("{magnet}");
        return Ok(());
    }

    launch(&magnet)?;
    println!("launched: {}", entry.title);
    Ok(())
}

fn launch(target: &str) -> Result<()> {
    use std::process::Command;
    let status = if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(target).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(target).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", target])
            .status()
    } else {
        bail!("don't know how to open URLs on this OS");
    };
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => bail!("opener exited with {s}"),
        Err(e) => bail!("failed to spawn opener: {e}"),
    }
}
