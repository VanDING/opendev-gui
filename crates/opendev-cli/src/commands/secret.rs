use opendev_secrets::{ChainedSecretStore, doctor::diagnose, migration::migrate_settings_json};
use std::sync::Arc;

pub async fn handle_secret_doctor() -> Result<(), String> {
    let config_dir = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .map(|h| h.join(".opendev"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let file_path = config_dir.join("secrets.age");

    let secrets: Arc<ChainedSecretStore> = Arc::new(ChainedSecretStore::new(Some(file_path), None));

    let results = diagnose(&*secrets).await;
    opendev_secrets::doctor::print_report(&results);
    Ok(())
}

pub async fn handle_secret_migrate() -> Result<(), String> {
    let config_dir = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .map(|h| h.join(".opendev"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let file_path = config_dir.join("secrets.age");
    let secrets = ChainedSecretStore::new(Some(file_path), None);

    // Check settings.json and migrate if needed
    let settings_path = config_dir.join("settings.json");
    if settings_path.exists() {
        match migrate_settings_json(&settings_path, &secrets).await {
            Ok(report) => {
                if !report.moved.is_empty() {
                    println!("✅ Migrated {} secrets:", report.moved.len());
                    for item in &report.moved {
                        println!("   • {}", item);
                    }
                } else {
                    println!("✅ No secrets needed migration.");
                }
                if !report.errors.is_empty() {
                    println!("⚠️  {} errors during migration:", report.errors.len());
                    for err in &report.errors {
                        println!("   • {}", err);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Migration failed: {}", e);
                return Err(e.to_string());
            }
        }
    } else {
        println!("ℹ️  No settings.json found at {:?}", settings_path);
    }

    Ok(())
}
