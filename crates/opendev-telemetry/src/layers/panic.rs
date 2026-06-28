//! Panic handler — writes crash dumps to ~/.opendev/crash/.
//! Extracted from opendev-cli/src/helpers.rs.

use std::io::Write;
use std::path::PathBuf;

/// Install a panic handler that writes crash reports to `~/.opendev/crash/`.
///
/// Calls the default panic hook after writing the report.
/// This should be called early in main(), before TelemetryGuard::init().
pub fn install_crash_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let mut report = String::new();
        report.push_str("OpenDev Crash Report\n");
        report.push_str(&format!("Timestamp: {}\n", chrono::Utc::now()));
        report.push_str(&format!("Version: {}\n\n", env!("CARGO_PKG_VERSION")));

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };
        report.push_str(&format!("Panic: {}\n", message));

        if let Some(location) = panic_info.location() {
            report.push_str(&format!(
                "Location: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        }

        report.push_str(&format!("\nBacktrace:\n{}\n", backtrace));

        // Write crash report
        let crash_dir = get_crash_dir();
        if let Some(crash_dir) = &crash_dir {
            let _ = std::fs::create_dir_all(crash_dir);
            let filename = format!("crash-{}.log", timestamp);
            let crash_path = crash_dir.join(&filename);
            let temp_suffix = uuid::Uuid::new_v4();
            let temp_path = crash_dir.join(format!(".{filename}.{temp_suffix}.tmp"));

            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }

            let success = opts.open(&temp_path).ok()
                .and_then(|mut f| f.write_all(report.as_bytes()).ok())
                .and_then(|_| std::fs::rename(&temp_path, &crash_path).ok())
                .is_some();

            if success {
                eprintln!(
                    "\nOpenDev crashed unexpectedly. A crash report has been saved to:\n  {}\n\nPlease include this file when reporting the issue.\n",
                    crash_path.display()
                );
            } else {
                eprintln!("\nOpenDev crashed unexpectedly. Failed to write crash report.\n");
            }
        }

        default_hook(panic_info);
    }));
}

fn get_crash_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok()
        .map(|h| PathBuf::from(h).join(".opendev").join("crash"))
}
