use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("akash")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Search Pattern")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input File(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("recursive")
                .help("Directory Search")
                .short("r")
                .long("recursive")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("count")
                .help("Count Occurrence")
                .short("c")
                .long("count")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("invertmatch")
                .help("Invert match")
                .short("v")
                .long("invert-match")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case-insensitive")
                .short("i")
                .long("insensitive")
                .takes_value(false),
        )
        .get_matches();

    let pat_string = matches.value_of("pattern").unwrap();
    let pattern = RegexBuilder::new(pat_string)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_| format!("Invalid pattern \"{}\"", pat_string))?;
    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invertmatch"),
        pattern,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}
