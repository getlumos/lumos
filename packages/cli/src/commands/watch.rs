// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Watch command - regenerate code on file changes

use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Watch mode: regenerate on file changes
pub fn run(schema_path: &Path, output_dir: Option<&Path>, lang: &str, target: &str) -> Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let schema_path = schema_path.to_path_buf();
    let output_dir_buf = output_dir.map(|p| p.to_path_buf());
    let lang_string = lang.to_string();
    let target_string = target.to_string();

    println!(
        "{:>12} {} for changes...",
        "Watching".cyan().bold(),
        schema_path.display()
    );
    println!("Press Ctrl+C to stop");
    println!();

    // Initial generation (no safety flags in watch mode)
    if let Err(e) = crate::commands::generate::run(
        &schema_path,
        output_dir,
        &lang_string,
        &target_string,
        false,
        false,
        false,
    ) {
        eprintln!("{}: {}", "error".red().bold(), e);
    }

    // Set up file watcher
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    watcher.watch(&schema_path, RecursiveMode::NonRecursive)?;

    // Get configurable debounce duration (default: 100ms)
    let debounce_ms = std::env::var("LUMOS_WATCH_DEBOUNCE")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&ms| ms <= 5000) // Max 5 seconds
        .unwrap_or(100);

    // Watch for changes
    loop {
        match rx.recv_timeout(Duration::from_millis(debounce_ms)) {
            Ok(_event) => {
                // Debounce: wait a bit for multiple rapid changes
                std::thread::sleep(Duration::from_millis(debounce_ms));

                // Drain any pending events
                while rx.try_recv().is_ok() {}

                println!();
                println!("{:>12} change detected", "Detected".yellow().bold());

                if let Err(e) = crate::commands::generate::run(
                    &schema_path,
                    output_dir_buf.as_deref(),
                    &lang_string,
                    &target_string,
                    false,
                    false,
                    false,
                ) {
                    eprintln!("{}: {}", "error".red().bold(), e);
                }

                println!();
                println!("{:>12} for changes...", "Watching".cyan().bold());
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Normal timeout, continue watching
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    Ok(())
}
