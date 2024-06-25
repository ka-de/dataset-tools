use dataset_tools_rs::walk_directory;
use regex::Regex;
use std::fs::File;
use std::io::{ self, BufRead, BufReader };
use std::path::Path;

fn main() -> io::Result<()> {
    let re = Regex::new(r"\b[¹²³⁴⁵⁶⁷⁸⁹]\b").unwrap();
    let dir = Path::new(r"C:\Users\kade\code\cringe.live\docs");

    walk_directory(dir, "md", |path| {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if re.is_match(&line) {
                println!("{}:{}: {}", path.display(), index + 1, line);
            }
        }

        Ok(())
    })?;

    Ok(())
}
