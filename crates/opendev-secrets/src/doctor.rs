//! Secret doctor — diagnose shadow-key issues.
//!
//! Scans all known secret keys and reports:
//! - Which secrets are in the keyring
//! - Which are overridden by env vars ("shadowed")
//! - Which only exist in env vars (not migrated)
//! - Which are in the deprecated AppConfig.api_key

use crate::key::SecretKey;
use crate::provider::SecretProvider;
use crate::store::SecretStore;

/// Status of a single secret key.
#[derive(Debug)]
pub struct SecretStatus {
    pub key: SecretKey,
    pub in_keyring: bool,
    pub env_var: Option<String>,
    pub env_set: bool,
    pub shadowed: bool, // true if in keyring AND env is set
    pub healthy: bool,  // true if in keyring and not shadowed (or explicitly opted-in)
}

/// Run the secret doctor on a SecretStore.
/// Returns a report of all known secrets and their shadow status.
pub async fn diagnose(secrets: &dyn SecretStore) -> Vec<SecretStatus> {
    let mut results = Vec::new();

    // Check all known LLM providers
    for &(provider, env_var) in SecretProvider::known_env_vars() {
        let key = SecretKey::llm(provider);
        let in_keyring = secrets.get(&key).await.ok().flatten().is_some();
        let env_set = std::env::var(env_var).map(|v| !v.is_empty()).unwrap_or(false);
        let shadowed = in_keyring && env_set;

        results.push(SecretStatus {
            key,
            in_keyring,
            env_var: Some(env_var.to_string()),
            env_set,
            shadowed,
            healthy: in_keyring && !shadowed,
        });
    }

    // Check Telegram bot token
    let tg_key = SecretKey::telegram();
    let tg_in_keyring = secrets.get(&tg_key).await.ok().flatten().is_some();
    let tg_env_set = std::env::var("TELEGRAM_BOT_TOKEN").map(|v| !v.is_empty()).unwrap_or(false);
    results.push(SecretStatus {
        key: tg_key,
        in_keyring: tg_in_keyring,
        env_var: Some("TELEGRAM_BOT_TOKEN".into()),
        env_set: tg_env_set,
        shadowed: tg_in_keyring && tg_env_set,
        healthy: tg_in_keyring && !tg_env_set,
    });

    results
}

/// Pretty-print the doctor report.
pub fn print_report(results: &[SecretStatus]) {
    println!("═══ OpenDev Secret Doctor ═══");
    println!();

    let healthy: Vec<_> = results.iter().filter(|r| r.healthy).collect();
    let shadowed: Vec<_> = results.iter().filter(|r| r.shadowed).collect();
    let not_migrated: Vec<_> = results.iter().filter(|r| !r.in_keyring && r.env_set).collect();

    if healthy.is_empty() && shadowed.is_empty() && not_migrated.is_empty() {
        println!(
            "✅ No secrets found. Use `opendev setup` or the settings UI to configure API keys."
        );
        println!();
        return;
    }

    if !healthy.is_empty() {
        println!("✅ Healthy (keyring only, no env override):");
        for s in &healthy {
            println!("   • {} → keyring", s.key);
        }
        println!();
    }

    if !shadowed.is_empty() {
        println!("⚠️  Shadowed (keyring has value but env var overrides):");
        for s in &shadowed {
            println!("   • {} (keyring)", s.key);
            println!("     ← env: {} is still set", s.env_var.as_ref().unwrap_or(&String::new()));
            println!(
                "     To fix: unset {} or use the keyring value",
                s.env_var.as_ref().unwrap_or(&String::new())
            );
        }
        println!();
    }

    if !not_migrated.is_empty() {
        println!("📦 Not migrated (only in env var, not in keyring):");
        for s in &not_migrated {
            println!(
                "   • {} → set only via env var {}",
                s.key,
                s.env_var.as_ref().unwrap_or(&String::new())
            );
        }
        println!("   Run `opendev secret migrate` to store them in the keyring.");
        println!();
    }

    // Summary
    println!("═══ Summary ═══");
    println!("  Healthy:       {}/{}", healthy.len(), results.len());
    println!("  Shadowed:      {}/{}", shadowed.len(), results.len());
    println!("  Not migrated:  {}/{}", not_migrated.len(), results.len());
    println!();
}
