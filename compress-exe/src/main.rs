use std::env;
use std::process::Command;
use std::fs;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 {
        &args[1]
    } else {
        ".\\target\\x86_64-pc-windows-msvc\\release\\"
    };

    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("exe") {
            let status = Command::new("upx").arg("--best").arg(path.clone()).status()?;
            if !status.success() {
                eprintln!("Failed to compress {}", path.display());
            }
        }
    }

    Ok(())
}
