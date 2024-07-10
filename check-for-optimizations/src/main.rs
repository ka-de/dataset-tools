// check-for-optimizations\src\main.rs

// This program searches for Cargo.toml files that have `profile.dev.opt-level = 3` and
// reports the files that do not.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::path::{ Path, PathBuf };
use dataset_tools::{ walk_directory, read_file_content };
use anyhow::{ Result, Context };
use toml::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

async fn check_cargo_toml(path: &Path) -> Result<bool> {
    let content = read_file_content(path.to_str().unwrap()).await.context("Failed to read file")?;
    let toml_value: Value = content.parse().context("Failed to parse TOML")?;

    let Some(profile) = toml_value.get("profile") else {
        return Ok(false);
    };

    // Check [profile.dev]
    let Some(dev) = profile.get("dev") else {
        return Ok(false);
    };
    if dev.get("opt-level") != Some(&Value::Integer(3)) {
        return Ok(false);
    }

    // Check [profile.dev.package."*"]
    let Some(dev_package) = dev.get("package").and_then(|p| p.get("*")) else {
        return Ok(false);
    };
    if
        dev_package.get("opt-level") != Some(&Value::Integer(3)) ||
        dev_package.get("codegen-units") != Some(&Value::Integer(1))
    {
        return Ok(false);
    }

    // Check [profile.release]
    let Some(release) = profile.get("release") else {
        return Ok(false);
    };
    if
        release.get("opt-level") != Some(&Value::Integer(3)) ||
        release.get("lto") != Some(&Value::Boolean(true)) ||
        release.get("codegen-units") != Some(&Value::Integer(1)) ||
        release.get("strip") != Some(&Value::Boolean(true))
    {
        return Ok(false);
    }

    Ok(true)
}

async fn process_target(target: &Path) -> Result<(), anyhow::Error> {
    let missing_configs = Arc::new(Mutex::new(Vec::new()));

    if target.is_file() && target.file_name().unwrap() == "Cargo.toml" {
        if !check_cargo_toml(target).await.unwrap_or(false) {
            missing_configs.lock().await.push(target.to_owned());
        }
    } else if target.is_dir() {
        let missing_configs_clone = Arc::clone(&missing_configs);
        walk_directory(target, "toml", move |path: PathBuf| {
            let missing_configs = Arc::clone(&missing_configs_clone);
            async move {
                if
                    path.file_name().unwrap() == "Cargo.toml" &&
                    !check_cargo_toml(&path).await.unwrap_or(false)
                {
                    missing_configs.lock().await.push(path);
                }
                Ok(())
            }
        }).await?;
    } else {
        println!("Invalid path: {}", target.display());
        return Ok(());
    }

    let missing_configs = missing_configs.lock().await;
    if missing_configs.is_empty() {
        println!("All Cargo.toml files contain the required configurations.");
    } else {
        println!("The following Cargo.toml files are missing the required configurations:");
        for file in missing_configs.iter() {
            println!("{}", file.display());
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 { PathBuf::from(&args[1]) } else { std::env::current_dir()? };

    process_target(&target).await?;

    Ok(())
}
