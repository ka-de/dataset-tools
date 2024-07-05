// search-for-superscript-numbers\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use crossterm::{ ExecutableCommand, style::Print };
use crossterm::style::{ Color, SetForegroundColor, ResetColor };
use regex::Regex;
use std::io::{ stdout, Write };
use tokio::fs::File;
use tokio::io::{ AsyncBufReadExt, BufReader };
use std::path::Path;
use dataset_tools::walk_directory;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let re = Regex::new(r"[¹²³⁴⁵⁶⁷⁸⁹]").unwrap();
    let dir = Path::new(r"C:\Users\kade\code\cringe.live\");

    walk_directory(dir, "md", |path| {
        let value = re.clone();
        async move {
            let file = File::open(&path).await?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            let mut index = 1;
            while let Some(line) = lines.next_line().await? {
                if let Some(mat) = value.find(&line) {
                    let prefix = &line[..mat.start()];
                    let match_str = mat.as_str();
                    let suffix = &line[mat.end()..];

                    stdout().execute(SetForegroundColor(Color::Magenta))?;
                    stdout().execute(Print(format!("{}:", path.display())))?;
                    stdout().execute(SetForegroundColor(Color::Green))?;
                    stdout().execute(Print(format!("{index}: ")))?;
                    stdout().execute(ResetColor)?;
                    stdout().execute(Print(prefix))?;
                    stdout().execute(SetForegroundColor(Color::Red))?;
                    stdout().execute(Print(match_str))?;
                    stdout().execute(ResetColor)?;
                    stdout().execute(Print(suffix))?;
                    stdout().write_all(b"\n")?;
                }
                index += 1;
            }

            Ok(())
        }
    }).await?;

    Ok(())
}
