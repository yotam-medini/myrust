use std::io::BufRead; // This "unlocks" .lines()
use std::fmt::Write; // This "unlocks" the ability to write! into a String
use clap::{Arg, Command, Error};
use unicode_properties::UnicodeGeneralCategory;

struct CliArgs {
    input_file: String,
}

fn parse_arguments() -> Result<CliArgs, Error> {
    let matches = Command::new("pdf select and clean margins")
        .version("0.1.0")
        .author("yotam.medin@gmail.com")
        .about("Report Non-Ascii characters in file")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("input.txt")
                .help("text input")
                .required(true),
        )
        .try_get_matches()?; // Use `try_get_matches` to return a Result

    let input_file = matches.get_one::<String>("input").unwrap().to_string();

    Ok(CliArgs {
        input_file,
    })
}

fn c_to_encoding_str(c: char) -> String {
    let mut buf = [0; 4]; 
    let encoded = c.encode_utf8(&mut buf);
    let mut hex_string = String::new();
    for b in encoded.as_bytes() {
        write!(hex_string, "{:02x} ", b).unwrap();
    }
    hex_string
}

fn print_non_ascii(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(args.input_file.clone())?;
    let reader = std::io::BufReader::new(file);

    for (ln, line_result) in reader.lines().enumerate() {
	let line = line_result?;
	for (col, c) in line.chars().enumerate() {
	    if !c.is_ascii() {
                let category = c.general_category();
                let group = c.general_category_group();
                println!("{:4}:{:3}  c={}, [{}], category={:?}, group={:?}",
                    ln, col, c, c_to_encoding_str(c), category, group);
            }
	}
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match parse_arguments() {
        Ok(args) => {
            print_non_ascii(&args)
        }
        Err(e) => {
            e.exit();
        }
    }
}
