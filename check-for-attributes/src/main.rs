// Turn clippy into a real bitch
#![warn(clippy::all, clippy::pedantic)]

use walkdir::{ WalkDir, DirEntry };
use regex::Regex;
use crossterm::{ style::{ Color, SetForegroundColor, ResetColor, Stylize }, ExecutableCommand };
use std::fs::File;
use std::io::{ self, BufRead };
use std::io::stdout;

// List of built-in attributes in Rust
const ATTRIBUTES: &[&str] = &[
    "cfg",
    "cfg_attr",
    "test",
    "ignore",
    "should_panic",
    //"derive",
    "automatically_derived",
    "macro_export",
    "macro_use",
    "proc_macro",
    "proc_macro_derive",
    "proc_macro_attribute",
    "allow",
    "warn",
    "deny",
    "forbid",
    "deprecated",
    "must_use",
    "diagnostic::on_unimplemented",
    "link",
    "link_name",
    "link_ordinal",
    "no_link",
    "repr",
    "crate_type",
    "no_main",
    "export_name",
    "link_section",
    "no_mangle",
    "used",
    "crate_name",
    "inline",
    "cold",
    "no_builtins",
    "target_feature",
    "track_caller",
    "instruction_set",
    "doc",
    "no_std",
    "no_implicit_prelude",
    "path",
    "recursion_limit",
    "type_length_limit",
    "panic_handler",
    "global_allocator",
    "windows_subsystem",
    "feature",
    "non_exhaustive",
    "debugger_visualizer",
];

fn is_target_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.path().ends_with("target")
}

fn main() -> io::Result<()> {
    let re = Regex::new(
        &format!(r"#\[\s*({})|#!\[\s*({})\]", ATTRIBUTES.join("|"), ATTRIBUTES.join("|"))
    );

    let regex = match re {
        Ok(regex) => regex,
        Err(e) => {
            // Convert regex::Error to std::io::Error
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !is_target_dir(e))
        .filter_map(Result::ok) {
        if entry.path().is_file() {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "rs" {
                    let file = File::open(path)?;
                    let reader = io::BufReader::new(file);

                    let mut lines: Vec<String> = Vec::new();
                    for line in reader.lines() {
                        lines.push(line?);
                    }

                    for (i, line) in lines.iter().enumerate() {
                        if regex.is_match(line) {
                            let start = if i >= 2 { i - 2 } else { 0 };
                            let end = if i + 3 < lines.len() { i + 3 } else { lines.len() };

                            stdout().execute(SetForegroundColor(Color::Magenta))?;
                            println!("{}:{}", path.display(), i + 1);
                            stdout().execute(ResetColor)?;

                            for (j, line) in lines[start..end]
                                .iter()
                                .enumerate()
                                .map(|(j, line)| (j + start, line)) {
                                if j == i {
                                    let highlighted = regex.replace_all(
                                        line,
                                        |caps: &regex::Captures| { format!("{}", caps[0].red()) }
                                    );
                                    println!("{highlighted}");
                                } else {
                                    println!("{line}");
                                }
                            }
                            println!(); // Extra newline for separation
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
