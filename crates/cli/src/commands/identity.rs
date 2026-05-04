use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use tt_identity::{DefaultStore, IdentityStore, LocalKeypair, npub::encode_npub, portable};
use tt_storage::Database;

use crate::fmt::short_id;

#[derive(Subcommand)]
pub enum IdentityCmd {
    /// Generate a new identity keypair (stored in the OS keyring with a file
    /// fallback). Refuses to overwrite an existing identity unless --force.
    Init(InitArgs),
    /// Show the local identity's npub and metadata.
    Show,
    /// Export the encrypted identity backup to a file.
    Export(ExportArgs),
    /// Import an encrypted identity backup, replacing the current one.
    Import(ImportArgs),
    /// Forget the local identity (does NOT remove published entries).
    Forget(ForgetArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Display name (optional) — purely cosmetic.
    #[arg(long)]
    pub name: Option<String>,
    /// Overwrite an existing identity.
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Path to write the encrypted backup to.
    pub path: std::path::PathBuf,
    /// Passphrase. If omitted, prompts on stdin (not implemented in Phase 2;
    /// pass --passphrase explicitly).
    #[arg(long)]
    pub passphrase: String,
}

#[derive(Args)]
pub struct ImportArgs {
    /// Path to the encrypted backup file.
    pub path: std::path::PathBuf,
    #[arg(long)]
    pub passphrase: String,
    /// Overwrite an existing identity.
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ForgetArgs {
    /// Skip the safety prompt.
    #[arg(long)]
    pub yes: bool,
}

pub fn run(cmd: IdentityCmd, db: &Database) -> Result<()> {
    match cmd {
        IdentityCmd::Init(a) => init(a, db),
        IdentityCmd::Show => show(db),
        IdentityCmd::Export(a) => export(a),
        IdentityCmd::Import(a) => import(a, db),
        IdentityCmd::Forget(a) => forget(a, db),
    }
}

pub fn load_keypair() -> Result<LocalKeypair> {
    let store = DefaultStore::new().context("init identity store")?;
    let seed = store
        .load()
        .context("load identity")?
        .context("identity not initialized; run `tt identity init`")?;
    Ok(LocalKeypair::from_seed(&seed))
}

fn init(args: InitArgs, db: &Database) -> Result<()> {
    let store = DefaultStore::new()?;
    if !args.force && store.load()?.is_some() {
        bail!(
            "an identity already exists. Use --force to overwrite (you will lose access to past contributions signed with the previous key)."
        );
    }
    let kp = LocalKeypair::generate();
    let seed = kp.seed();
    store.store(&seed)?;
    db.put_local_identity(&kp.public_bytes(), args.name.as_deref())?;
    println!(
        "identity created\n  npub: {}\n  short id: {}",
        kp.npub(),
        short_id(&kp.npub()[5..]) // skip "npub1" prefix in the short label
    );
    println!(
        "\n⚠  Sauvegarde immédiatement ta clé avec `tt identity export <path> --passphrase <pp>`."
    );
    println!("   Sans backup, perdre ta machine = perdre l'identité.");
    Ok(())
}

fn show(db: &Database) -> Result<()> {
    let store = DefaultStore::new()?;
    let identity = match db.get_local_identity()? {
        Some(i) => i,
        None => {
            println!("no identity yet. Create one with `tt identity init`.");
            return Ok(());
        }
    };
    let npub = encode_npub(&identity.pubkey)?;
    let in_keyring = store.load()?.is_some();
    println!("local identity");
    println!("  npub        : {npub}");
    println!("  pubkey hex  : {}", hex::encode(identity.pubkey.0));
    if let Some(n) = &identity.display_name {
        println!("  display name: {n}");
    }
    println!("  created at  : {}", identity.created_at.to_rfc3339());
    println!(
        "  key storage : {}",
        if in_keyring {
            "OS keyring or file"
        } else {
            "MISSING (you have a public key in DB but no private key — broken state)"
        }
    );
    Ok(())
}

fn export(args: ExportArgs) -> Result<()> {
    let kp = load_keypair()?;
    let blob = portable::export(&kp.seed(), &args.passphrase)?;
    if let Some(parent) = args.path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.path, &blob)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&args.path, std::fs::Permissions::from_mode(0o600))?;
    }
    println!(
        "identity exported to {} ({} bytes)",
        args.path.display(),
        blob.len()
    );
    Ok(())
}

fn import(args: ImportArgs, db: &Database) -> Result<()> {
    let store = DefaultStore::new()?;
    if !args.force && store.load()?.is_some() {
        bail!("an identity already exists. Use --force to overwrite.");
    }
    let blob = std::fs::read(&args.path)?;
    let seed = portable::import(&blob, &args.passphrase)?;
    let kp = LocalKeypair::from_seed(&seed);
    store.store(&seed)?;
    db.put_local_identity(&kp.public_bytes(), None)?;
    println!("identity imported");
    println!("  npub: {}", kp.npub());
    Ok(())
}

fn forget(args: ForgetArgs, db: &Database) -> Result<()> {
    if !args.yes {
        bail!(
            "this removes the private key from the keyring/file and clears the\n\
             local identity row. Re-run with --yes to confirm."
        );
    }
    let store = DefaultStore::new()?;
    store.delete()?;
    db.clear_local_identity()?;
    println!("identity forgotten.");
    Ok(())
}
