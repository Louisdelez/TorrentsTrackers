use anyhow::{Result, bail};
use chrono::Utc;
use clap::{Args, Subcommand};
use tt_core::{ConflictStrategy, DedupStrategy, Pool, PoolFilters, PoolId};
use tt_storage::Database;

use crate::fmt::short_id;

#[derive(Subcommand)]
pub enum PoolCmd {
    /// Create a new pool combining one or more sources.
    Create(CreateArgs),
    /// List configured pools.
    List,
    /// Delete a pool by id (prefix match allowed).
    Remove(RemoveArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// Display name of the pool.
    pub name: String,
    /// Source ids (prefix match) to include in this pool.
    #[arg(long, num_args = 1.., value_delimiter = ',')]
    pub sources: Vec<String>,
}

#[derive(Args)]
pub struct RemoveArgs {
    pub id: String,
}

pub fn run(cmd: PoolCmd, db: &Database) -> Result<()> {
    match cmd {
        PoolCmd::Create(a) => create(a, db),
        PoolCmd::List => list(db),
        PoolCmd::Remove(a) => remove(a, db),
    }
}

fn create(args: CreateArgs, db: &Database) -> Result<()> {
    let all_sources = db.list_sources()?;
    let mut members = Vec::new();
    for prefix in &args.sources {
        let matches: Vec<_> = all_sources
            .iter()
            .filter(|s| s.id.0.to_string().starts_with(prefix))
            .collect();
        match matches.len() {
            0 => bail!("no source matches prefix '{prefix}'"),
            1 => members.push(matches[0].id),
            n => bail!("{n} sources match prefix '{prefix}'; be more specific."),
        }
    }
    if members.is_empty() {
        bail!("a pool needs at least one source. Use --sources <id> [<id> ...]");
    }
    let pool = Pool {
        id: PoolId::new(),
        name: args.name,
        description: None,
        members,
        filters: PoolFilters::default(),
        dedup_strategy: DedupStrategy::default(),
        conflict_strategy: ConflictStrategy::default(),
        created_at: Utc::now(),
    };
    db.insert_pool(&pool)?;
    println!(
        "created pool {} '{}' with {} source(s)",
        short_id(&pool.id.0.to_string()),
        pool.name,
        pool.members.len()
    );
    Ok(())
}

fn list(db: &Database) -> Result<()> {
    let pools = db.list_pools()?;
    if pools.is_empty() {
        println!("no pools yet. Create one with `tt pool create <name> --sources <ids...>`.");
        return Ok(());
    }
    println!("{} pool(s):", pools.len());
    for p in pools {
        println!(
            "  {}  {:30}  {} source(s)",
            short_id(&p.id.0.to_string()),
            p.name,
            p.members.len()
        );
    }
    Ok(())
}

fn remove(args: RemoveArgs, db: &Database) -> Result<()> {
    let pools = db.list_pools()?;
    let matches: Vec<_> = pools
        .iter()
        .filter(|p| p.id.0.to_string().starts_with(&args.id))
        .collect();
    let p = match matches.len() {
        0 => bail!("no pool matches prefix '{}'", args.id),
        1 => matches[0].clone(),
        n => bail!("{n} pools match prefix '{}'", args.id),
    };
    db.delete_pool(p.id)?;
    println!(
        "removed pool {} '{}'",
        short_id(&p.id.0.to_string()),
        p.name
    );
    Ok(())
}
