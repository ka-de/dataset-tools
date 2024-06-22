
/**
 * This program uses the Aho-Corasick algorithm to find patterns in a given string (haystack).
 * The patterns and the haystack are defined in the main function.
 *
 * The AhoCorasick::new function is used to create a new AhoCorasick structure that will be used to find the patterns in the haystack.
 * If a match is found, the details of the match (pattern id, start and end indices) are stored in a vector.
 *
 * The program also uses the crossterm crate to color the matched patterns in the terminal output.
 * A HashMap is used to map each pattern id to a color. If a pattern id does not have a corresponding color in the map, a random color is chosen from a predefined array of colors.
 *
 * The program then iterates over the matches, and for each match, it prints the part of the haystack before the match in the default color, the match in its corresponding color, and then resets the color.
 *
 * Finally, the program prints the remaining part of the haystack after the last match.
 */

use aho_corasick::AhoCorasick;
use crossterm::{ style::{ Color, Print, ResetColor, SetForegroundColor }, ExecutableCommand };
use rand::Rng;
use std::collections::HashMap;
use std::io::stdout;

fn main() {
    let patterns = &["Samwise", "Sam"];
    let haystack = "Samwise";
    let ac = AhoCorasick::new(patterns).unwrap();

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
