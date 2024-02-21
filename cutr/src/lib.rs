use crate::Extract::*;
use clap::{App, Arg};
use csv::Position;
use std::{ops::Range};
use std::error::Error;


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
    use super::parse_pos;
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
        assert_eq!(
            res.unwrap_err().to_string(),
            "illegal list value: \"+1\"",
            );

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "illegal list value: \"+1-2\"",
            );

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "illegal list value: \"1-+2\"",
            );

        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");


        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");


        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "illegal list value: \"1-a\"",
        );

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "illegal list value: \"a-1\"",
            );


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
                .multiple(true)
            )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .help("Selected bytes")
                .short("b")
                .long("bytes")
                .conflicts_with("chars")
                .conflicts_with("fields")
            )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .help("Selected characters")
                .short("c")
                .long("chars")
                .conflicts_with("bytes")
                .conflicts_with("fields")
            )
        .arg(
            Arg::with_name("delim")
                .value_name("DELIMITER")
                .help("Field delimiter")
                .short("d")
                .long("delim")
                .default_value("\t")
            )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .help("Selected fields")
                .short("f")
                .long("fields")
                .conflicts_with("chars")
                .conflicts_with("bytes")
            ).get_matches();
    
    let extract = {
        if let Some(str_range) = matches.value_of("chars") {
            Extract::Chars(parse_pos(str_range)?) 
        } else if let Some(str_range) = matches.value_of("bytes") {
            Extract::Bytes(parse_pos(str_range)?)
        } else if let Some(str_range) = matches.value_of("files") {
            Extract::Fields(parse_pos(str_range)?)
        } else {
            return Err(From::from("Must have --fields, --bytes, or --chars"));
        }
    };

    Ok(Config{
        files: matches.values_of_lossy("files").unwrap(),
        delimiter: matches.value_of("delim").unwrap().as_bytes()[0].clone(),
        extract,
    })
}


fn is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_digit(10))
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let mut pos_list = PositionList::new();
    for str_range in range.split(",") {
        let start_str: String = str_range.chars().take_while(|&c| c != '-').collect();
        let end_str: String = str_range.chars().skip_while(|&c| c != '-').skip(1).collect();

        println!("{}", str_range);
        println!("{} {}", start_str.len(), end_str.len());
        println!("{:?}", start_str.parse::<usize>());
        println!();

        if !is_numeric(&start_str) || (!is_numeric(&end_str) && end_str.len() != 0) ||
            (str_range.contains('-') && end_str.len() == 0){
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
            start -= 1
        } else {
            end = start;
            start = end - 1;
        }
        
        if start >= end {
            return Err(format!(
                        "First number in range ({}) must be lower than the second number ({})",
                        start,
                        end
                    ).into());
        }

        pos_list.push(Range::<usize> {
            start,
            end,
        });
    }
    Ok(pos_list)
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", &config);
    Ok(())
}
