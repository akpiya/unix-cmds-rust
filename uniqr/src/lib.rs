use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("akash")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input File")
                .default_value("-")
            )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output File")
            )
        .arg(
            Arg::with_name("count")
                .short("c")
                .help("Show counts")
                .long("count")
                .takes_value(false)
            )
        .get_matches();

    Ok(Config {
        in_file: matches.value_of_lossy("in_file").unwrap().to_string(),
        out_file: matches.value_of_lossy("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}


pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file)
        .map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut prev_line = String::new();
    let mut count = 0;
    let mut output = String::new();
    loop {
        let bytes = file.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        if prev_line.trim() != line.trim() {
            if count > 0 {
                if config.count {
                    output.push_str(&format!("{:>4} {}", count, prev_line));
                } else{
                    output.push_str(&format!("{}", prev_line));
                }
            } 
            prev_line = line.clone();
            count = 0;
        } 
        
        count += 1;
        line.clear();
    }
    if count > 0 {
        if config.count {
            output.push_str(&format!("{:>4} {}", count, prev_line));
        } else {
            output.push_str(&format!("{}", prev_line));
        }
    }
    match config.out_file {
        None => print!("{}", output),
        Some(filename) => {
            let mut f = File::create(filename)?;
            let _res = f.write(output.as_bytes());
        },
    }
    Ok(())
}






