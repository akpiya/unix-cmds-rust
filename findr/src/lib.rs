use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::{collections::btree_map::Entry, error::Error};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}


pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("akash")
        .about("Rust find")
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .help("Search paths")
                .default_value(".")
                .multiple(true)
            )
        .arg(
            Arg::with_name("names")
                .value_name("NAME")
                .short("n")
                .long("name")
                .help("Name")
                .takes_value(true)
                .multiple(true)
            )
        .arg(
            Arg::with_name("types")
                .value_name("TYPE")
                .short("t")
                .long("type")
                .help("Entry type") 
                .possible_values(&["f", "d", "l"])
                .multiple(true)
                .takes_value(true)
            )
        .get_matches();

    let names = matches.values_of_lossy("names").map(|pat| {
        pat.into_iter().map(|name| {
            Regex::new(&name)
                .map_err(|_| format!("Invalid --name \"{}\"", name))
        }).collect::<Result<Vec<_>,_>>()
    }).transpose()?.unwrap_or_default();

    let entry_types = matches.values_of_lossy("types")
        .map(|vals| {
            vals.iter()
                .map(|val| match val.as_str() {
                    "d" => Dir,
                    "f" => File,
                    "l" => Link,
                    _ => unreachable!("Invalid Type"),
                })
                .collect()
        }).unwrap_or_default();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        names,
        entry_types, 
    })
}


pub fn run(config: Config) -> MyResult<()> {
    // println!("{:?}", config);
    for path in config.paths {
        for entry in WalkDir::new(path).follow_links(true) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(val) => {
                    //println!("{:?}", val);
                    let filename = val.file_name().to_string_lossy();
                    // println!("{}", filename);
                    
                    if config.names.iter().any(|regex| regex.is_match(&filename)) || config.names.len() == 0 {
                        let filetype: Option<EntryType>;
                        if val.file_type().is_file() {
                            filetype = Some(EntryType::File);
                        } else if val.file_type().is_dir() {
                            filetype = Some(EntryType::Dir);
                        } else if val.file_type().is_symlink() {
                            filetype = Some(EntryType::Link);
                        } else{
                            filetype = None;
                        }

                        if let Some(ftype) = filetype {
                            if config.entry_types.contains(&ftype) || config.entry_types.len() == 0 {
                                println!("{}", val.path().display());
                            }
                        }
                    }
                },
            }
        }
    }
    Ok(())
}
