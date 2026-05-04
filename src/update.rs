use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

const CURRENT: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECS: u64 = 86_400; // re-check crates.io once per day

/// Checks crates.io for a newer version. Returns `Some(version)` if one exists.
/// Results are cached at `~/.cache/fhevm-forge/update_check` to avoid a network
/// hit on every invocation.
pub async fn check() -> Option<String> {
    let cache_file = cache_path()?;
    let latest = match read_cache(&cache_file) {
        Some(v) => v,
        None => {
            let v = fetch_latest().await.ok()?;
            let _ = write_cache(&cache_file, &v);
            v
        }
    };
    if is_newer(&latest, CURRENT) { Some(latest) } else { None }
}

// ── crates.io fetch ────────────────────────────────────────────────────────────

async fn fetch_latest() -> anyhow::Result<String> {
    #[derive(serde::Deserialize)]
    struct Resp { #[serde(rename = "crate")] krate: Krate }
    #[derive(serde::Deserialize)]
    struct Krate { newest_version: String }

    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .user_agent(format!(
            "fhevm-forge/{} (https://github.com/Godbrand0/fhevm-forge)",
            CURRENT
        ))
        .build()?
        .get("https://crates.io/api/v1/crates/fhevm-forge")
        .send()
        .await?
        .json::<Resp>()
        .await?;

    Ok(resp.krate.newest_version)
}

// ── cache (version + timestamp, two lines) ────────────────────────────────────

fn cache_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join(".cache/fhevm-forge");
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join("update_check"))
}

fn read_cache(path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut lines = content.lines();
    let version = lines.next()?.to_string();
    let ts: u64 = lines.next()?.parse().ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    if now.saturating_sub(ts) < CHECK_INTERVAL_SECS { Some(version) } else { None }
}

fn write_cache(path: &PathBuf, version: &str) -> std::io::Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    fs::write(path, format!("{}\n{}\n", version, now))
}

// ── semver comparison (x.y.z only) ────────────────────────────────────────────

fn is_newer(candidate: &str, current: &str) -> bool {
    parse_semver(candidate) > parse_semver(current)
}

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let mut parts = v.trim_start_matches('v').splitn(3, '.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}
