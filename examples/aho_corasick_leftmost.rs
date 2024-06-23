/// * This program uses the Aho-Corasick algorithm for multiple pattern matching.
///  * It takes a list of patterns and a string (haystack) to search in.
///  * The AhoCorasick::builder() is used to build an AhoCorasick structure with the MatchKind set to LeftmostFirst.
///  * This means that in case of overlapping matches, the match occurring first in the string will be returned.
///  *
///  * In this specific example, the patterns are "Samwise" and "Sam", and the haystack is "Samwise".
///  * Since "Samwise" occurs before "Sam" in the haystack, the whole string "Samwise" will be highlighted.
///  *
///  * The matches are stored in a vector, and each match is a tuple containing the pattern ID and the start and end indices of the match in the haystack.
///  *
///  * A HashMap is used to map each pattern ID to a color. The colors are randomly chosen from a predefined array.
///  *
///  * The program then iterates over the matches, and for each match, it prints the part of the haystack before the match in the default color,
///  * then it prints the match in its assigned color, and finally resets the color.
///  *
///  * After all matches have been processed, it prints the rest of the haystack in the default color.

use aho_corasick::{ AhoCorasick, MatchKind };
use crossterm::{ style::{ Color, Print, ResetColor, SetForegroundColor }, ExecutableCommand };
use rand::Rng;
use std::collections::HashMap;
use std::io::stdout;

fn main() {
    let patterns = &["Samwise", "Sam"];
    let haystack = "Samwise";
    let ac = AhoCorasick::builder().match_kind(MatchKind::LeftmostFirst).build(patterns).unwrap();

    let mut matches = vec![];
    if let Some(mat) = ac.find(haystack) {
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
