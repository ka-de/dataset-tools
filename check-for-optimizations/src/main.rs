// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::path::{ Path, PathBuf };
use dataset_tools::{ walk_directory, read_file_content };
use anyhow::{ Result, Context, anyhow };
use toml::Value;

fn check_cargo_toml(path: &Path) -> Result<bool> {
    let content = read_file_content(path.to_str().unwrap()).context("Failed to read file")?;
    let toml_value: Value = content.parse()?;

    let profile = toml_value.get("profile").ok_or_else(|| anyhow!("No profile section found"))?;

    // Check [profile.dev]
    let dev = profile.get("dev").ok_or_else(|| anyhow!("No profile.dev section found"))?;
    if dev.get("opt-level") != Some(&Value::Integer(3)) {
        return Ok(false);
    }

    // Check [profile.dev.package."*"]
    let dev_package = dev
        .get("package")
        .and_then(|p| p.get("*"))
        .ok_or_else(|| anyhow!("No profile.dev.package.\"*\" section found"))?;
    if
        dev_package.get("opt-level") != Some(&Value::Integer(3)) ||
        dev_package.get("codegen-units") != Some(&Value::Integer(1))
    {
        return Ok(false);
    }

    // Check [profile.release]
    let release = profile
        .get("release")
        .ok_or_else(|| anyhow!("No profile.release section found"))?;
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

fn process_target(target: &Path) -> Result<(), anyhow::Error> {
    let mut missing_configs = Vec::new();

    if target.is_file() && target.file_name().unwrap() == "Cargo.toml" {
        if !check_cargo_toml(target)? {
            missing_configs.push(target.to_owned());
        }
    } else if target.is_dir() {
        walk_directory(target, "toml", |path| {
            if path.file_name().unwrap() == "Cargo.toml" {
                match check_cargo_toml(path) {
                    Ok(false) => {
                        missing_configs.push(path.to_owned());
                        Ok(())
                    }
                    Ok(true) => Ok(()),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                }
            } else {
                Ok(())
            }
        })?;
    } else {
        println!("Invalid path: {}", target.display());
        return Ok(());
    }

    if missing_configs.is_empty() {
        println!("All Cargo.toml files contain the required configurations.");
    } else {
        println!("The following Cargo.toml files are missing the required configurations:");
        for file in &missing_configs {
            println!("{}", file.display());
        }
    }

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 { PathBuf::from(&args[1]) } else { std::env::current_dir()? };

    process_target(&target)?;

    Ok(())
}
