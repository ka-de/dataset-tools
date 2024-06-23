/// * This program demonstrates the use of the Aho-Corasick algorithm for multiple pattern matching.
///  * It takes a list of patterns and a text (haystack) as input. The Aho-Corasick algorithm is used
///  * to find all occurrences of the patterns in the text.
///  *
///  * Each pattern found in the text is highlighted with a unique color. The colors are randomly
///  * assigned from a predefined list. The same pattern will always be highlighted with the same color.
///  *
///  * The `crossterm` crate is used for handling colored output to the terminal, and the `rand` crate
///  * is used for generating random colors.
///  *
///  * # Example
///  *
///  * Given the patterns ["apple", "maple", "Snapple"] and the text "Nobody in the snapple industry
///  * likes maple in their apple flavored Snapple.", the program will print the text to the terminal
///  * with each occurrence of the patterns highlighted in a unique color.
///  *
///  * # Panics
///  *
///  * The program will panic if it fails to write to the standard output.
///  *
///  * # Safety
///  *
///  * This function is safe as long as the standard output can be written to.

use aho_corasick::AhoCorasick;
use crossterm::{ style::{ Color, Print, ResetColor, SetForegroundColor }, ExecutableCommand };
use rand::Rng;
use std::collections::HashMap;
use std::io::stdout;

fn main() {
    let patterns = &["apple", "maple", "Snapple"];
    let haystack = "Nobody in the snapple industry likes maple in their apple flavored Snapple.";
    let ac = AhoCorasick::new(patterns).unwrap();

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
