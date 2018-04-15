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
    eprintln!("Usage: invisible-text{}", extra);
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

    // Creating a clipboard should always succeed(?)
    let mut clipboard = clipboard::ClipboardContext::new().unwrap();

    match mode {
        Mode::Hide => {
            // Find arg[2] and print usage data if it isn't found
            let data = match args.next() {
                Some(data) => data,
                None => return print_usage(" hide [ascii-data]"),
            };

            // Try to convert the string to invisible text. This fails if any of the
            // characters are not in ('\x20'...'\x7F')
            let new_data: String = match data.chars().map(to_invisible).collect() {
                Some(new_data) => new_data,
                None => return invalid_hide(),
            };

            // Copy the contents to the clipboard and print an error if it fails
            match clipboard.set_contents(new_data) {
                Ok(()) => println!("Copied to clipboard!"),
                Err(e) => return eprintln!("Error: {}", e),
            }
        }

        Mode::Find => {
            // Try to get the contents from the clipboard. If it fails we'll
            // assume it's because the clipboard didn't contain textual data.
            let data = match clipboard.get_contents() {
                Ok(data) => data,
                Err(_) => return invalid_find(),
            };

            // Find all of the invisible characters from the string, stripping away
            // other codepoints.
            let new_data: String = data.chars().flat_map(from_invisible).collect();
            println!("{:?}", new_data);
        }

        Mode::Intersperse => {
            // Get the message and data to encode
            let (message, data) = match (args.next(), args.next()) {
                (Some(message), Some(data)) => (message, data),
                _ => return print_usage(" intersperse [message] [ascii-data]"),
            };

            // We encode the data at unicode word-boundary positions, so we
            // need to know how many segments there are.
            let segment_count = message.split_word_bounds().count();
            if segment_count < 2 {
                eprintln!("Can't intersperse data in a message with less than 2 words");
            }
            // We're only placing data on the inner positions
            let intersperse_points = segment_count - 1;
            // See how many we put at each point
            let chunk_size = (data.len() + intersperse_points - 1) / intersperse_points;

            // Get our two iterators
            let mut segments = message.split_word_bounds();
            let mut data_chunks = &mut data.chars().flat_map(to_invisible).fuse();

            // Allocate a buffer and insert the very first segment
            let mut new_data = String::with_capacity(message.len() + data.len() * 4);
            new_data.push_str(segments.next().unwrap());

            // For the rest of the segments, insert a data chunk prefix
            for segment in segments {
                new_data.extend(data_chunks.take(chunk_size));
                new_data.push_str(segment);
            }

            // Save the data to the clipboard
            match clipboard.set_contents(new_data) {
                Ok(()) => println!("Copied to clipboard!"),
                Err(e) => return eprintln!("Error: {}", e),
            }
        }
    }
}
