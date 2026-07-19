use clap::{Arg, ArgAction, Command};
use cornetti::redis::{confs::RedisDBConfig, services::RedisDBService};
use redis::AsyncCommands;

fn app_id() -> String {
    std::env::var("APP_ID").unwrap_or_else(|_| "app".to_string())
}

fn tenant_id() -> String {
    std::env::var("APP_TENANT_ID").unwrap_or_else(|_| "DEFAULT".to_string())
}

fn confirm_yes() -> bool {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "s" | "si")
}

async fn connect() -> Result<redis::aio::MultiplexedConnection, Box<dyn std::error::Error>> {
    let cfg = RedisDBConfig::from_env();
    let redis = RedisDBService::new(&cfg)?;
    Ok(redis.client().get_multiplexed_async_connection().await?)
}

fn print_keys(keys: &[String]) {
    if keys.is_empty() {
        println!("  (vuoto)");
        return;
    }
    for k in keys {
        println!("  - {}", k);
    }
}

// ---------------------------------------------------------------------------
// list-permissions
// ---------------------------------------------------------------------------

fn list_permissions_cmd() -> Command {
    Command::new("list-permissions")
        .about("Elenca tutte le cache permessi")
}

async fn cmd_list_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let pattern = format!("{}:permissions:*", app_id());
    let mut conn = connect().await?;
    let keys: Vec<String> = conn.keys(&pattern).await?;
    println!("Cache permessi ({}):", pattern);
    print_keys(&keys);
    println!("Totale: {}", keys.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// show-user
// ---------------------------------------------------------------------------

fn show_user_cmd() -> Command {
    Command::new("show-user")
        .about("Mostra i permessi cached di un utente")
        .arg(Arg::new("email").help("Email dell'utente").required(true))
}

async fn cmd_show_user(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let email = args.get_one::<String>("email").unwrap();
    let key = format!("{}:permissions:{}", app_id(), email);
    let mut conn = connect().await?;
    let exists: bool = conn.exists(&key).await?;
    if !exists {
        println!("Nessuna cache permessi trovata per '{}'.", email);
        return Ok(());
    }
    let value: String = conn.get(&key).await?;
    let ttl: i64 = conn.ttl(&key).await?;
    println!("Chiave: {}", key);
    println!("TTL: {}", if ttl < 0 { "nessuno".to_string() } else { format!("{}s", ttl) });
    println!("Valore:\n{}", serde_json::to_string_pretty(&value).unwrap_or(value));
    Ok(())
}

// ---------------------------------------------------------------------------
// flush-permissions
// ---------------------------------------------------------------------------

fn flush_permissions_cmd() -> Command {
    Command::new("flush-permissions")
        .about("Cancella TUTTE le cache permessi")
        .arg(Arg::new("yes").short('y').long("yes").help("Salta conferma").action(ArgAction::SetTrue))
}

async fn cmd_flush_permissions(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let pattern = format!("{}:permissions:*", app_id());
    let mut conn = connect().await?;
    let keys: Vec<String> = conn.keys(&pattern).await?;
    if keys.is_empty() {
        println!("Nessuna cache permessi da eliminare.");
        return Ok(());
    }
    println!("Trovate {} cache permessi:", keys.len());
    print_keys(&keys);
    if !args.get_flag("yes") {
        print!("Eliminare {} chiavi? (s/N): ", keys.len());
        std::io::Write::flush(&mut std::io::stdout())?;
        if !confirm_yes() {
            println!("Annullato.");
            return Ok(());
        }
    }
    let count: usize = conn.del(&keys).await?;
    println!("Eliminate {} chiavi.", count);
    Ok(())
}

// ---------------------------------------------------------------------------
// list-tokens
// ---------------------------------------------------------------------------

fn list_tokens_cmd() -> Command {
    Command::new("list-tokens")
        .about("Elenca token auth e refresh cached")
}

async fn cmd_list_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let tenant = tenant_id();
    let app = app_id();
    let mut conn = connect().await?;

    let auth_pat = format!("{}:{}:auth:*", tenant, app);
    let refresh_pat = format!("{}:{}:refresh:*", tenant, app);

    let auth_keys: Vec<String> = conn.keys(&auth_pat).await?;
    let refresh_keys: Vec<String> = conn.keys(&refresh_pat).await?;

    println!("Token auth ({}):", auth_pat);
    print_keys(&auth_keys);
    println!("Token refresh ({}):", refresh_pat);
    print_keys(&refresh_keys);
    println!("Totale: {}", auth_keys.len() + refresh_keys.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// flush-tokens
// ---------------------------------------------------------------------------

fn flush_tokens_cmd() -> Command {
    Command::new("flush-tokens")
        .about("Cancella TUTTI i token auth e refresh")
        .arg(Arg::new("yes").short('y').long("yes").help("Salta conferma").action(ArgAction::SetTrue))
}

async fn cmd_flush_tokens(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let tenant = tenant_id();
    let app = app_id();
    let mut conn = connect().await?;

    let auth_pat = format!("{}:{}:auth:*", tenant, app);
    let refresh_pat = format!("{}:{}:refresh:*", tenant, app);

    let auth_keys: Vec<String> = conn.keys(&auth_pat).await?;
    let refresh_keys: Vec<String> = conn.keys(&refresh_pat).await?;

    let all: Vec<String> = [auth_keys, refresh_keys].concat();
    if all.is_empty() {
        println!("Nessun token da eliminare.");
        return Ok(());
    }
    println!("Trovati {} token:", all.len());
    print_keys(&all);
    if !args.get_flag("yes") {
        print!("Eliminare {} token? (s/N): ", all.len());
        std::io::Write::flush(&mut std::io::stdout())?;
        if !confirm_yes() {
            println!("Annullato.");
            return Ok(());
        }
    }
    let count: usize = conn.del(&all).await?;
    println!("Eliminati {} token.", count);
    Ok(())
}

// ---------------------------------------------------------------------------
// list-sessions
// ---------------------------------------------------------------------------

fn list_sessions_cmd() -> Command {
    Command::new("list-sessions")
        .about("Elenca sessioni cached")
}

async fn cmd_list_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let tenant = tenant_id();
    let app = app_id();
    let mut conn = connect().await?;

    let pattern = format!("{}:{}:sessions:*", tenant, app);
    let keys: Vec<String> = conn.keys(&pattern).await?;
    println!("Sessioni ({}):", pattern);
    print_keys(&keys);
    println!("Totale: {}", keys.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// flush-sessions
// ---------------------------------------------------------------------------

fn flush_sessions_cmd() -> Command {
    Command::new("flush-sessions")
        .about("Cancella TUTTE le sessioni")
        .arg(Arg::new("yes").short('y').long("yes").help("Salta conferma").action(ArgAction::SetTrue))
}

async fn cmd_flush_sessions(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let tenant = tenant_id();
    let app = app_id();
    let mut conn = connect().await?;

    let pattern = format!("{}:{}:sessions:*", tenant, app);
    let keys: Vec<String> = conn.keys(&pattern).await?;
    if keys.is_empty() {
        println!("Nessuna sessione da eliminare.");
        return Ok(());
    }
    println!("Trovate {} sessioni:", keys.len());
    print_keys(&keys);
    if !args.get_flag("yes") {
        print!("Eliminare {} sessioni? (s/N): ", keys.len());
        std::io::Write::flush(&mut std::io::stdout())?;
        if !confirm_yes() {
            println!("Annullato.");
            return Ok(());
        }
    }
    let count: usize = conn.del(&keys).await?;
    println!("Eliminate {} sessioni.", count);
    Ok(())
}

// ---------------------------------------------------------------------------
// flush-user
// ---------------------------------------------------------------------------

fn flush_user_cmd() -> Command {
    Command::new("flush-user")
        .about("Cancella permessi cached di un utente")
        .arg(Arg::new("email").help("Email dell'utente").required(true))
}

async fn cmd_flush_user(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let email = args.get_one::<String>("email").unwrap();
    let key = format!("{}:permissions:{}", app_id(), email);
    let mut conn = connect().await?;
    let exists: bool = conn.exists(&key).await?;
    if !exists {
        println!("Nessuna cache permessi per '{}'.", email);
        return Ok(());
    }
    let _: usize = conn.del(&key).await?;
    println!("Cache permessi eliminata per '{}'.", email);
    Ok(())
}

// ---------------------------------------------------------------------------
// flush-all
// ---------------------------------------------------------------------------

fn flush_all_cmd() -> Command {
    Command::new("flush-all")
        .about("Cancella TUTTE le chiavi Redis del DB corrente")
        .arg(Arg::new("yes").short('y').long("yes").help("Salta conferma").action(ArgAction::SetTrue))
}

async fn cmd_flush_all(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = connect().await?;
    let keys: Vec<String> = conn.keys("*").await?;
    if keys.is_empty() {
        println!("Nessuna chiave presente nel DB corrente.");
        return Ok(());
    }
    println!("ATTENZIONE: verranno eliminate TUTTE le {} chiavi:", keys.len());
    print_keys(&keys);
    if !args.get_flag("yes") {
        print!("Eliminare TUTTE le {} chiavi? (s/N): ", keys.len());
        std::io::Write::flush(&mut std::io::stdout())?;
        if !confirm_yes() {
            println!("Annullato.");
            return Ok(());
        }
    }
    let count: usize = conn.del(&keys).await?;
    println!("Eliminate {} chiavi.", count);
    Ok(())
}

// ---------------------------------------------------------------------------
// Command builder & dispatch
// ---------------------------------------------------------------------------

pub fn redis_cmd() -> clap::Command {
    Command::new("redis")
        .about("Gestione cache Redis")
        .subcommand_required(true)
        .subcommand(list_permissions_cmd())
        .subcommand(show_user_cmd())
        .subcommand(flush_permissions_cmd())
        .subcommand(flush_user_cmd())
        .subcommand(list_tokens_cmd())
        .subcommand(flush_tokens_cmd())
        .subcommand(list_sessions_cmd())
        .subcommand(flush_sessions_cmd())
        .subcommand(flush_all_cmd())
}

pub async fn dispatch(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand() {
        Some(("list-permissions", _)) => cmd_list_permissions().await,
        Some(("show-user", a)) => cmd_show_user(a).await,
        Some(("flush-permissions", a)) => cmd_flush_permissions(a).await,
        Some(("flush-user", a)) => cmd_flush_user(a).await,
        Some(("list-tokens", _)) => cmd_list_tokens().await,
        Some(("flush-tokens", a)) => cmd_flush_tokens(a).await,
        Some(("list-sessions", _)) => cmd_list_sessions().await,
        Some(("flush-sessions", a)) => cmd_flush_sessions(a).await,
        Some(("flush-all", a)) => cmd_flush_all(a).await,
        _ => Ok(()),
    }
}
