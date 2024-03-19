use clap::{App, Arg};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .version("0.1.0")
        .author("akash")
        .about("Rust comm")
        .arg(
            Arg::with_name("col1")
                .short("1")
                .help("Suppress column 1")
                .takes_value(false)
            )
        .arg(
            Arg::with_name("col2")
                .short("2")
                .help("Suppress column 2")
                .takes_value(false)
            )
        .arg(
            Arg::with_name("col3")
                .short("3")
                .help("Suppress column 3")
                .takes_value(false)
            )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Makes matches case insensitive")
                .takes_value(false)
            )
        .get_matches();
}
