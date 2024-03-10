use crate::Extract::*;
use clap::{App, Arg};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::ops::Range;

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

#[cfg(test)]
mod unit_tests {
    use super::extract_bytes;
    use super::extract_chars;
    use super::extract_fields;
    use super::parse_pos;
    use csv::StringRecord;

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"])
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_parse_pos() {
        assert!(parse_pos("").is_err());

        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than the second number (1)"
        );

        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("akash")
        .about("Rust cut")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input File(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .help("Selected bytes")
                .short("b")
                .long("bytes")
                .conflicts_with("chars")
                .conflicts_with("fields"),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .help("Selected characters")
                .short("c")
                .long("chars")
                .conflicts_with("bytes")
                .conflicts_with("fields"),
        )
        .arg(
            Arg::with_name("delim")
                .value_name("DELIMITER")
                .help("Field delimiter")
                .short("d")
                .long("delim")
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .help("Selected fields")
                .short("f")
                .long("fields")
                .conflicts_with("chars")
                .conflicts_with("bytes"),
        )
        .get_matches();

    let extract = {
        if let Some(str_range) = matches.value_of("chars") {
            Extract::Chars(parse_pos(str_range)?)
        } else if let Some(str_range) = matches.value_of("bytes") {
            Extract::Bytes(parse_pos(str_range)?)
        } else if let Some(str_range) = matches.value_of("fields") {
            Extract::Fields(parse_pos(str_range)?)
        } else {
            return Err(From::from("Must have --fields, --bytes, or --chars"));
        }
    };
    let delim = matches.value_of("delim").unwrap();

    if delim.as_bytes().len() != 1 {
        return Err(format!("--delim \"{}\" must be a single byte", delim).into());
    }

    let a = Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        delimiter: delim.as_bytes()[0].clone(),
        extract,
    });
    return a;
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_digit(10))
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let mut pos_list = PositionList::new();
    for str_range in range.split(",") {
        let start_str: String = str_range.chars().take_while(|&c| c != '-').collect();
        let end_str: String = str_range
            .chars()
            .skip_while(|&c| c != '-')
            .skip(1)
            .collect();

        if !is_numeric(&start_str)
            || (!is_numeric(&end_str) && end_str.len() != 0)
            || (str_range.contains('-') && end_str.len() == 0)
        {
            return Err(format!("illegal list value: \"{}\"", str_range).into());
        }

        let mut start: usize = match start_str.parse() {
            Ok(n) if n > 0 => n,
            Ok(_) => return Err(format!("illegal list value: \"{}\"", start_str).into()),
            Err(_) => return Err(format!("illegal list value: \"{}\"", str_range).into()),
        };

        let mut end: usize = 0;
        if end_str.len() != 0 {
            end = match end_str.parse() {
                Ok(n) if n > 0 => n,
                Ok(_) => return Err(format!("illegal list value: \"{}\"", end_str).into()),
                Err(_) => return Err(format!("illegal list value: \"{}\"", str_range).into()),
            };

            if start >= end {
                return Err(format!(
                    "First number in range ({}) must be lower than the second number ({})",
                    start, end
                )
                .into());
            }
        } else {
            end = start;
            start = end;
        }

        pos_list.push(Range::<usize> {
            start: start - 1,
            end,
        });
    }
    Ok(pos_list)
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let mut ret = String::new();
    let line_char = line.chars().collect::<Vec<_>>();
    for range in char_pos {
        let start_opt = if range.start < line_char.len() {
            Some(range.start)
        } else {
            None
        };
        let end = if range.end < line_char.len() {
            range.end
        } else {
            line_char.len()
        };

        if let Some(start) = start_opt {
            ret.push_str(
                &line_char[start..end]
                    .iter()
                    .map(|&x| x.to_string())
                    .collect::<String>(),
            );
        }
    }
    ret
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let mut ret = String::new();
    let line_bytes = line.bytes().collect::<Vec<_>>();

    for range in byte_pos {
        let start_opt = if range.start < line_bytes.len() {
            Some(range.start)
        } else {
            None
        };
        let end = if range.end < line_bytes.len() {
            range.end
        } else {
            line_bytes.len()
        };

        if let Some(start) = start_opt {
            let selected_bytes = &line_bytes[start..end];
            ret.push_str(
                String::from_utf8_lossy(selected_bytes)
                    .into_owned()
                    .as_str(),
            );
        }
    }
    ret
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    let mut ret = Vec::new();
    for range in field_pos {
        let start_opt = if range.start < record.len() {
            Some(range.start)
        } else {
            None
        };
        let end = if range.end < record.len() {
            range.end
        } else {
            record.len()
        };

        if let Some(start) = start_opt {
            for i in start..end {
                ret.push(record.get(i).unwrap().to_string());
            }
        }
    }
    ret
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => match &config.extract {
                Fields(pos) => {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(config.delimiter)
                        .has_headers(false)
                        .from_reader(file);
                    let mut wtr = WriterBuilder::new()
                        .delimiter(config.delimiter)
                        .from_writer(io::stdout());
                    for record in reader.records() {
                        let record = record?;
                        wtr.write_record(extract_fields(&record, pos))?;
                    }
                }
                Bytes(pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, pos));
                    }
                }
                Chars(pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, pos));
                    }
                }
            },
        }
    }
    Ok(())
}
