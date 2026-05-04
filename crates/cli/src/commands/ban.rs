use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use tt_identity::npub;
use tt_storage::Database;

use crate::commands::identity::load_keypair;
use crate::fmt::short_id;

#[derive(Subcommand)]
pub enum BanCmd {
    /// Ban a contributor (by npub or hex pubkey) in a given source.
    Add(BanAddArgs),
    /// Lift a ban.
    Remove(BanRemoveArgs),
    /// List bans for a source.
    List(BanListArgs),
}

#[derive(Args)]
pub struct BanAddArgs {
    /// Contributor pubkey: `npub1...` or 64-char hex.
    pub pubkey: String,
    /// Source id prefix.
    #[arg(long = "in")]
    pub source: String,
    /// Reason (optional).
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Args)]
pub struct BanRemoveArgs {
    pub pubkey: String,
    #[arg(long = "in")]
    pub source: String,
}

#[derive(Args)]
pub struct BanListArgs {
    #[arg(long = "in")]
    pub source: String,
}

pub fn run(cmd: BanCmd, db: &Database) -> Result<()> {
    match cmd {
        BanCmd::Add(a) => add(a, db),
        BanCmd::Remove(a) => remove(a, db),
        BanCmd::List(a) => list(a, db),
    }
}

fn parse_pubkey(s: &str) -> Result<tt_core::PublicKeyBytes> {
    if s.starts_with("npub1") {
        Ok(npub::decode_npub(s)?)
    } else {
        let bytes = hex::decode(s).context("pubkey is not valid hex")?;
        if bytes.len() != 32 {
            bail!("pubkey hex must be 64 chars (got {})", bytes.len() * 2);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(tt_core::PublicKeyBytes(arr))
    }
}

fn resolve_source(db: &Database, prefix: &str) -> Result<tt_core::Source> {
    let sources = db.list_sources()?;
    let m: Vec<_> = sources
        .into_iter()
        .filter(|s| s.id.0.to_string().starts_with(prefix))
        .collect();
    match m.len() {
        0 => bail!("no source matches prefix '{prefix}'"),
        1 => Ok(m.into_iter().next().unwrap()),
        n => bail!("{n} sources match prefix '{prefix}'"),
    }
}

fn add(args: BanAddArgs, db: &Database) -> Result<()> {
    let pk = parse_pubkey(&args.pubkey)?;
    let source = resolve_source(db, &args.source)?;
    let kp = load_keypair()?;
    db.add_ban(source.id, &pk, args.reason.as_deref(), &kp.public_bytes())?;
    println!(
        "banned {} in {} '{}'",
        short_id(&hex::encode(pk.0)),
        short_id(&source.id.0.to_string()),
        source.display_name
    );
    Ok(())
}

fn remove(args: BanRemoveArgs, db: &Database) -> Result<()> {
    let pk = parse_pubkey(&args.pubkey)?;
    let source = resolve_source(db, &args.source)?;
    db.remove_ban(source.id, &pk)?;
    println!(
        "unbanned {} in {}",
        short_id(&hex::encode(pk.0)),
        source.display_name
    );
    Ok(())
}

fn list(args: BanListArgs, db: &Database) -> Result<()> {
    let source = resolve_source(db, &args.source)?;
    let bans = db.list_bans(source.id)?;
    if bans.is_empty() {
        println!("no bans in {}", source.display_name);
        return Ok(());
    }
    println!("{} ban(s) in {}:", bans.len(), source.display_name);
    for b in bans {
        let reason = b.reason.as_deref().unwrap_or("(no reason)");
        println!(
            "  {}  {}  by {}",
            short_id(&hex::encode(b.pubkey.0)),
            reason,
            short_id(&hex::encode(b.banned_by.0))
        );
    }
    Ok(())
}
