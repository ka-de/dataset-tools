use aho_corasick::AhoCorasick;
use crossterm::{ style::{ Color, Print, ResetColor, SetForegroundColor }, ExecutableCommand };
use rand::Rng;
use std::collections::HashMap;
use std::io::stdout;

fn main() {
    let patterns = &["fox", "brown", "quick"];
    let replace_with = &["sloth", "grey", "slow"];

    let haystack = "The quick brown fox.";
    let ac = AhoCorasick::new(patterns).unwrap();

    let mut result = vec![];
    ac.try_stream_replace_all(haystack.as_bytes(), &mut result, replace_with).expect(
        "try_stream_replace_all failed"
    );

    let result_str = String::from_utf8(result).unwrap();

    let mut matches = vec![];
    for mat in ac.find_iter(&result_str) {
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
        stdout.execute(Print(&result_str[last_end..*start])).unwrap();
        stdout.execute(SetForegroundColor(*color)).unwrap();
        stdout.execute(Print(&result_str[*start..*end])).unwrap();
        stdout.execute(ResetColor).unwrap();
        last_end = *end;
    }
    stdout.execute(Print(&result_str[last_end..])).unwrap();
}
