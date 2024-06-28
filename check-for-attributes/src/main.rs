// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_rust_files, read_lines };
use regex::Regex;
use crossterm::{ style::{ Color, SetForegroundColor, ResetColor, Stylize }, ExecutableCommand };
use std::{ io::{ self, stdout }, path::Path };

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
    //"must_use",
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

fn main() -> io::Result<()> {
    let re = Regex::new(
        &format!(r"#\[\s*({})|#!\[\s*({})\]", ATTRIBUTES.join("|"), ATTRIBUTES.join("|"))
    ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    walk_rust_files(Path::new("."), |path| {
        let lines = read_lines(path)?;
        for (line_number, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                let start = line_number.saturating_sub(3);
                let end = (line_number + 2).min(lines.len());

                stdout().execute(SetForegroundColor(Color::Magenta))?;
                println!("{}:{}", path.display(), line_number + 1); // Add 1 because enumeration starts at 0
                stdout().execute(ResetColor)?;

                for (i, line) in lines[start..end].iter().enumerate() {
                    if i + start == line_number {
                        let highlighted = re.replace_all(line, |caps: &regex::Captures| {
                            format!("{}", caps[0].red())
                        });
                        println!("{highlighted}");
                    } else {
                        println!("{line}");
                    }
                }
                println!();
            }
        }
        Ok(())
    })
}
