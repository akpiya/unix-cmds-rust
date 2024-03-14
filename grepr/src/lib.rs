use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::io::{self,BufReader,BufRead};
use walkdir::{DirEntry, WalkDir};
use std::fs::{self, File, metadata};
use std::collections::HashMap;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        //Rejects directory without recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt"
            ]
        );

        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}

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

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut filenames: Vec<MyResult<String>> = Vec::new();
    for path in paths {
        if path.eq("-") {
            filenames.push(Ok(path.to_string())); 
        } else {
            match metadata(path) {
                Ok(metadata) => {
                    if metadata.is_file() {
                        filenames.push(Ok(path.clone()));
                    } else if metadata.is_dir() {
                        if recursive {
                            for entry in WalkDir::new(path)
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file()) {
                                
                                filenames.push(Ok(entry.path().display().to_string()));
                            }
                        } else {
                            filenames.push(Err(From::from(format!(
                                "{} is a directory",
                                path
                            ))));
                        }
                    }
                },
                Err(e) => {
                    filenames.push(Err(From::from(format!("{}: {}", path, e))));
                }
            }
        }
    }
    filenames
}

pub fn run(config: Config) -> MyResult<()> {
    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}", e),
                Ok(file) => {
                    let matches = find_lines(
                        file,
                        &config.pattern,
                        config.invert_match,
                    );
                    match matches {
                        Err(e) => eprintln!("{}", e),
                        Ok(string) => {
                            if config.count {
                                let len = string.iter().filter(|&e| !e.eq("")).collect::<Vec<_>>().len();
                                if config.files.len() > 1 || config.recursive{
                                    println!("{}:{}",filename,len);
                                } else {
                                    println!("{}", len);
                                }

                            } else if config.files.len() > 1 || config.recursive {
                                string.iter().for_each(|e| if !e.eq("") {print!("{}:{}", filename, e);})
                            } else if config.files.len() == 1 {
                                string.iter().for_each(|e| if !e.eq("") {print!("{}", e);})
                            }
                        }
                    }
                }
            },
        };
    }
    Ok(())
}


fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}


fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool
) -> MyResult<Vec<String>> {
    let mut matches = vec![];
    let mut line = String::new();

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if pattern.is_match(&line) ^ invert_match {
            matches.push(std::mem::take(&mut line));
        }
        line.clear();
    }
    Ok(matches)
}