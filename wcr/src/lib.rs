use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {

    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0; 
    let mut string = String::new();

    while let Ok(res) = file.read_line(&mut string){
        if res > 0 && (!string.eq("\n") || !string.eq("\r\n")) {
            num_lines += 1;
            num_words += string.split_whitespace().count();
            num_bytes += res; 
            num_chars += string.chars().count();
            string.clear();
        } else{
            break;
        }
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}


pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("akash")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input File(s)")
                .multiple(true)
                .default_value("-")
            )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .help("Print Line Count?")
                .short("l")
                .long("lines")
                .takes_value(false)
            )
        .arg(
            Arg::with_name("words")
                .value_name("WORDS")
                .help("Print Word Count?")
                .short("w")
                .long("words")
                .takes_value(false)
            )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .help("Print byte count?")
                .short("c")
                .long("bytes")
                .takes_value(false)
                .conflicts_with("chars")
            )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .help("Print char count?")
                .short("m")
                .long("chars")
                .takes_value(false)
                .conflicts_with("bytes")
            ).get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if !lines && !words && !bytes && !chars {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}


pub fn run(config: Config) -> MyResult<()> {
    let mut line_sum = 0;
    let mut word_sum = 0;
    let mut byte_sum = 0;
    let mut char_sum = 0;
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if let Ok(counts) = count(file) {
                    if config.lines {
                        print!("{:8}", counts.num_lines);
                        line_sum += counts.num_lines;
                    }
                    if config.words {
                        print!("{:8}", counts.num_words);
                        word_sum += counts.num_words;
                    }
                    if config.bytes {
                        print!("{:8}", counts.num_bytes);
                        byte_sum += counts.num_bytes;
                    }
                    if config.chars {
                        print!("{:8}", counts.num_chars);
                        char_sum += counts.num_chars;
                    }
                    if !config.files[0].eq("-") {
                        println!(" {}", filename);
                    } else {
                        println!();
                    }
                }
            },
        }
    }
    if config.files.len() > 1 {
        if config.lines {
            print!("{:8}", line_sum);
        }
        if config.words {
            print!("{:8}", word_sum);
        }
        if config.bytes {
            print!("{:8}", byte_sum);
        }
        if config.chars {
            print!("{:8}", char_sum);
        }
        println!(" total")
    }

    
    Ok(())
}
