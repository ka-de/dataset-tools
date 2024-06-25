// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools_rs::walk_rust_files;
use std::process::exit;
use regex::Regex;
use crossterm::style::{ Color, Print, ResetColor, SetForegroundColor };
use crossterm::ExecutableCommand;
use std::io;

fn main() -> io::Result<()> {
    let re = Regex::new(r"\.unwrap\(\)").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut found_unwrap = false;

    walk_rust_files(".", |path, line_number, line| {
        if re.is_match(line) {
            found_unwrap = true;
            let parts: Vec<&str> = line.split(".unwrap()").collect();
            std::io
                ::stdout()
                .execute(SetForegroundColor(Color::Magenta))?
                .execute(Print(format!("{}:{}:", path.display(), line_number)))?
                .execute(ResetColor)?
                .execute(Print(parts[0]))?
                .execute(SetForegroundColor(Color::Red))?
                .execute(Print(".unwrap()"))?
                .execute(ResetColor)?
                .execute(Print(parts[1]))?
                .execute(Print("\n"))?;
        }
        Ok(())
    })?;

    if found_unwrap {
        exit(1);
    }

    Ok(())
}
