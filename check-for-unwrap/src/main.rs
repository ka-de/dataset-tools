use std::process::exit;
use std::fs::File;
use std::io::{ self, BufRead };
use walkdir::WalkDir;
use regex::Regex;
use crossterm::style::{ Color, Print, ResetColor, SetForegroundColor };
use crossterm::ExecutableCommand;

fn main() -> io::Result<()> {
    // Create a Regex instance for matching ".unwrap()"
    let re = match Regex::new(r"\.unwrap\(\)") {
        Ok(re) => re, // If successful, store the Regex instance
        Err(e) => {
            // If there's an error creating the Regex instance
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    let mut found_unwrap = false; // Flag to track if any ".unwrap()" occurrences are found

    // Iterate over entries in the current directory and subdirectories
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok) // Keep only successful entries
        .filter(|e|
            // Keep only entries with a ".rs" extension (Rust source files)
            e
                .path()
                .extension()
                .map_or(false, |ext| ext == "rs")
        ) {
        let path = entry.path(); // Get the file path
        let display = path.display(); // Get the displayable path

        let file = File::open(&path)?; // Open the file
        let reader = io::BufReader::new(file); // Create a BufReader for efficient line reading

        // Enumerate lines in the file with line numbers
        for (num, line) in reader.lines().enumerate() {
            let l = line?; // Get the line as a string

            // Check if the line matches the ".unwrap()" pattern
            if re.is_match(&l) {
                found_unwrap = true; // Set the flag to true

                // Split the line into parts before and after ".unwrap()"
                let parts: Vec<&str> = l.split(".unwrap()").collect();

                // Print the line with colored highlights
                std::io
                    ::stdout()
                    .execute(SetForegroundColor(Color::Magenta))
                    ? // Set color to magenta
                    .execute(Print(format!("{}:{}:", display, num + 1)))
                    ? // Print file path and line number
                    .execute(ResetColor)
                    ? // Reset color
                    .execute(Print(parts[0]))
                    ? // Print part before ".unwrap()"
                    .execute(SetForegroundColor(Color::Red))
                    ? // Set color to red
                    .execute(Print(".unwrap()"))
                    ? // Print ".unwrap()"
                    .execute(ResetColor)
                    ? // Reset color
                    .execute(Print(parts[1]))
                    ? // Print part after ".unwrap()"
                    .execute(Print("\n"))?; // Print newline
            }
        }
    }

    // If any ".unwrap()" occurrences were found, exit with a non-zero status code
    if found_unwrap {
        exit(1);
    }

    Ok(()) // Return Ok(()) if no errors occurred
}
