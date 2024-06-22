
/**
 * This program uses the Aho-Corasick algorithm to find and highlight multiple patterns in a given text.
 *
 * The `main` function does the following:
 * 1. Defines a list of patterns and a text string (haystack) to search in.
 * 2. Builds an AhoCorasick struct with the patterns, enabling case-insensitive search.
 * 3. Iterates over the matches found in the haystack, storing each match's pattern ID and start/end indices.
 * 4. Defines a color map to associate each pattern ID with a color.
 * 5. Defines an array of colors to be used.
 * 6. Iterates over the matches, printing the text from the last match's end to the current match's start in default color.
 * 7. Prints the matched text in the color associated with its pattern ID.
 * 8. Resets the color back to default after printing each match.
 * 9. Finally, prints the remaining text after the last match in default color.
 *
 * The color for each pattern ID is chosen randomly from the defined color array.
 * If a color has already been chosen for a pattern ID, it is reused for subsequent matches of the same pattern.
 *
 * This program requires the `aho_corasick`, `crossterm`, and `rand` crates.
 */

use aho_corasick::AhoCorasick;
use crossterm::{ style::{ Color, Print, ResetColor, SetForegroundColor }, ExecutableCommand };
use rand::Rng;
use std::collections::HashMap;
use std::io::stdout;

fn main() {
    let patterns = &["apple", "maple", "snapple"];
    let haystack = "Nobody in the snapple industry likes maple in their apple flavored Snapple.";
    let ac = AhoCorasick::builder().ascii_case_insensitive(true).build(patterns).unwrap();

    let mut matches = vec![];
    for mat in ac.find_iter(haystack) {
        matches.push((mat.pattern().as_usize(), mat.start(), mat.end()));
    }

    let mut color_map: HashMap<usize, Color> = HashMap::new();

    #[rustfmt::skip]
    let colors = [ Color::Red, Color::Green, Color::Yellow, Color::Blue,
                   Color::Magenta, Color::Cyan, Color::DarkBlue, Color::DarkCyan,
                   Color::DarkGreen, Color::DarkMagenta, Color::DarkRed,
                   Color::DarkYellow,
    ];

    let mut rng = rand::thread_rng();
    let mut stdout = stdout();
    let mut last_end = 0;

    for (id, start, end) in matches.iter() {
        let color = color_map.entry(*id).or_insert(colors[rng.gen_range(0..colors.len())]);
        stdout.execute(Print(&haystack[last_end..*start])).unwrap();
        stdout.execute(SetForegroundColor(*color)).unwrap();
        stdout.execute(Print(&haystack[*start..*end])).unwrap();
        stdout.execute(ResetColor).unwrap();
        last_end = *end;
    }
    stdout.execute(Print(&haystack[last_end..])).unwrap();
}
