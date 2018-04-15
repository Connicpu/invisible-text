extern crate clipboard;
extern crate unicode_segmentation;

use std::char;

use clipboard::ClipboardProvider;
use unicode_segmentation::UnicodeSegmentation;

fn to_invisible(c: char) -> Option<char> {
    match c {
        '\x20'...'\x7F' => char::from_u32(0xE0000 + c as u32),
        _ => None,
    }
}

fn from_invisible(c: char) -> Option<char> {
    match c {
        '\u{E0020}'...'\u{E007F}' => char::from_u32(c as u32 - 0xE0000),
        _ => None,
    }
}

enum Mode {
    Hide,
    Find,
    Intersperse,
}

fn print_usage(extra: &str) {
    eprintln!("Usage: invisible-json{}", extra);
}

fn invalid_hide() {
    eprintln!("Invalid data: can't hide non-ascii json strings");
}

fn invalid_find() {
    eprintln!("Invalid data: there was no invisible text in your clipboard");
}

fn main() {
    let mut args = std::env::args().skip(1);

    let mode = match args.next().as_ref().map(|s| &s[..]) {
        Some("hide") => Mode::Hide,
        Some("find") => Mode::Find,
        Some("intersperse") => Mode::Intersperse,
        _ => return print_usage(" [hide|find|intersperse]"),
    };

    let mut clipboard = clipboard::ClipboardContext::new().unwrap();

    match mode {
        Mode::Hide => {
            let data = match args.next() {
                Some(data) => data,
                None => return print_usage(" hide [json-data]"),
            };

            let new_data: String = match data.chars().map(to_invisible).collect() {
                Some(new_data) => new_data,
                None => return invalid_hide(),
            };

            match clipboard.set_contents(new_data) {
                Ok(()) => println!("Copied to clipboard!"),
                Err(e) => return eprintln!("Error: {}", e),
            }
        }

        Mode::Find => {
            let data = match clipboard.get_contents() {
                Ok(data) => data,
                Err(_) => return invalid_find(),
            };

            let new_data: String = data.chars().flat_map(from_invisible).collect();
            eprintln!("{:?}", new_data);
        }

        Mode::Intersperse => {
            let (message, data) = match (args.next(), args.next()) {
                (Some(message), Some(data)) => (message, data),
                _ => return print_usage(" intersperse [message] [json-data]"),
            };

            let boundary_count = message.split_word_bounds().count();
            if boundary_count < 2 {
                eprintln!("Can't intersperse data with less than ")
            }
            let intersperse_points = boundary_count - 1;
            let chunk_size = (data.len() + intersperse_points - 1) / intersperse_points;

            let mut segments = message.split_word_bounds();
            let mut data_chunks = &mut data.chars().flat_map(to_invisible).fuse();

            let mut new_data = String::with_capacity(message.len() + data.len() * 4);
            new_data.push_str(segments.next().unwrap());

            for segment in segments {
                new_data.extend(data_chunks.take(chunk_size));
                new_data.push_str(segment);
            }

            match clipboard.set_contents(new_data) {
                Ok(()) => println!("Copied to clipboard!"),
                Err(e) => return eprintln!("Error: {}", e),
            }
        }
    }
}
