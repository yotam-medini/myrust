// use std;
// use std::fs::File;
use std::io::BufRead; // This "unlocks" .lines()
use clap::{Arg, Command, Error};
// use std::fmt;
// use unicode_properties::UnicodeEmoji;
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

fn noascii(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
   println!("noascii: {}", args.input_file);
    let file = std::fs::File::open(args.input_file.clone())?;
    let reader = std::io::BufReader::new(file);

    for (ln, line_result) in reader.lines().enumerate() {
	let line = line_result?;
	// println!("Line {}", ln);
	for (col, c) in line.chars().enumerate() {
	    if !c.is_ascii() {
                let category = c.general_category();
                let group = c.general_category_group();
                println!("{}:{} c={}, category={:?}, group={:?}", ln, col, c, category, group);
            }
	}
	// println!(" ");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Call the parsing function and handle its result.
    match parse_arguments() {
        Ok(args) => {
            // If parsing was successful, proceed with the application logic.
            noascii(&args)
        }
        Err(e) => {
            // If parsing failed, print the error and exit gracefully.
            e.exit();
        }
    }
}
