use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("akash")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input File(s)")
                .multiple(true)
                .default_value("-")
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .help("Number of bytes")
                .takes_value(true)
                .conflicts_with("lines")
        ).get_matches();
    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: matches
            .value_of("lines")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal line count -- {}", e))?.unwrap(),
        bytes: matches
            .value_of("bytes")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal byte count -- {}", e))?,
    })
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());
    
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

pub fn run(config: Config) -> MyResult<()> { 
    let length = config.files.len();
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut reader) => {
                if length > 1 {
                    println!("==> {} <==", filename);
                }
                match config.bytes {
                    // Print the corresponding number of bytes
                    Some(num_bytes) => {
                        let mut buf = vec![0u8; num_bytes];
                        let bytes = reader.read_exact(&mut buf);
                        if bytes == 0 {
                            break;
                        }
                        let s = String::from_utf8_lossy(&buf);
                        print!("{}", s);
                    },
                    // Print corresponding number of lines
                    None => {
                        for _line_num in 0..config.lines {
                            let mut string = String::new();
                            if let Ok(_) = reader.read_line(&mut string) {
                                print!("{}", string);
                            }
                        }
                    },
                }
            },
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
