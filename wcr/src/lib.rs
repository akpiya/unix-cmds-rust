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
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(_) => println!("Opened {}", filename),
        }
    }
    Ok(())
}




