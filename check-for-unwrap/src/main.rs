// check-for-unwrap\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_rust_files, read_lines };
use std::{ process::exit, sync::Arc, io, path::Path };
use regex::Regex;
use crossterm::style::{ Color, Print, ResetColor, SetForegroundColor };
use crossterm::ExecutableCommand;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> io::Result<()> {
    let re = Arc::new(
        Regex::new(r"\.unwrap\(\)").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    );
    let found_unwrap = Arc::new(Mutex::new(false));

    walk_rust_files(Path::new("."), |path| {
        let re = Arc::clone(&re);
        let found_unwrap = Arc::clone(&found_unwrap);
        async move {
            let lines = read_lines(&path).await?;
            for (line_number, line) in lines.into_iter().enumerate() {
                if re.is_match(&line) {
                    let mut found_unwrap = found_unwrap.lock().await;
                    *found_unwrap = true;
                    let parts: Vec<&str> = line.split(".unwrap()").collect();
                    std::io
                        ::stdout()
                        .execute(SetForegroundColor(Color::Magenta))?
                        .execute(Print(format!("{}:{}:", path.display(), line_number + 1)))?
                        .execute(ResetColor)?
                        .execute(Print(parts[0]))?
                        .execute(SetForegroundColor(Color::Red))?
                        .execute(Print(".unwrap()"))?
                        .execute(ResetColor)?
                        .execute(Print(parts[1]))?
                        .execute(Print("\n"))?;
                }
            }
            Ok(())
        }
    }).await?;

    let found_unwrap = found_unwrap.lock().await;
    if *found_unwrap {
        exit(1);
    }

    Ok(())
}
