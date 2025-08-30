use clap::{Arg, Command, Error};

// The CliArgs struct holds the parsed and validated command-line arguments.
// This provides a clean interface for the main application logic.
struct CliArgs {
    input_file: String,
    output_file: String,
    selections: Vec<String>,
}

#[repr(usize)]
#[derive(Debug)]
enum Side {Left, Bottom, Right, Top, N, }

#[derive(Clone, Default, Debug)]
struct Selection {
    page_number : u32,
    margin_width: [u32; Side::N as usize],
}

impl Selection {
    fn new_or_default(s: &str, default_selection: &Selection) -> Result<Self, String> {
        let mut err_msg : String = String::new();
        let ss : Vec<String> = s.split(':').map(|s| s.to_string()).collect();
        let sslen = ss.len();
        println!("[{:?}] ss={:?}", sslen, ss);
        if sslen < 1 || 5 < sslen {
            err_msg = format!("Number of colon-separated values in {} is {}, must be within [1,5]",
                s, sslen);
        }
        let mut ret : Selection = default_selection.clone();
        if err_msg.is_empty() {
            match ss[0].parse::<u32>() {
                Ok(num) => { ret.page_number = num },
                Err(e) => { err_msg = e.to_string() },
            }
        }
        let mut i : usize = 0;
        while err_msg.is_empty() && i + 1 < sslen {
            if !ss[i + 1].is_empty() {
                match ss[i + 1].parse::<u32>() {
                    Ok(num) => { ret.margin_width[i] = num },
                    Err(e) => { err_msg = e.to_string() },
                }
            }
            i += 1;
        }
        if err_msg.is_empty() {
            Ok(ret)
        } else {
            Err(err_msg)
        }
    }
}

fn is_valid_selection(val: &str) -> Result<String, String> {
    let ss : Vec<String> = val.split(':').map(|s| s.to_string()).collect();
    let sslen = ss.len();
    println!("[{:?}] ss={:?}", sslen, ss);
    let mut err_msg : String = String::new();
    if sslen < 1 || 5 < sslen {
        err_msg = format!("Number of values in {} is {}, must be within [1,5]",
            val, sslen);
    }
    println!("err_msg={}", err_msg);
    let default_selection = Selection{ page_number: 0, margin_width: [0, 0, 0, 0], };
    match Selection::new_or_default(val, &default_selection) {
        Ok(_) => Ok(val.to_string()),
        Err(err_msg) => Err(err_msg),
    }
}

// This function is dedicated to parsing the command-line arguments.
// It returns a `Result` to allow the caller to handle parsing failures.
fn parse_arguments() -> Result<CliArgs, Error> {
    let matches = Command::new("My CLI App")
        .version("1.0")
        .author("You")
        .about("A simple application that processes files.")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Sets the input file to use")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output file to write to")
                .required(true),
        )
        .arg(
            Arg::new("selection")
                .short('s')
                .long("selection")
                .value_name("ITEM")
                .help("Select a specific item for processing (can be used multiple times)")
                .action(clap::ArgAction::Append)
                .required(true)
                .value_parser(is_valid_selection), // This now uses our new parsing function
        )
        .try_get_matches()?; // Use `try_get_matches` to return a Result

    // Extract the values and build the `CliArgs` struct.
    // `.unwrap()` is now safe because `try_get_matches` would have already
    // returned an error if these were missing or invalid.
    let input_file = matches.get_one::<String>("input").unwrap().to_string();
    let output_file = matches.get_one::<String>("output").unwrap().to_string();

    let selections: Vec<String> = matches.get_many::<String>("selection")
                                        .unwrap()
                                        .map(|s| s.to_string())
                                        .collect();

    Ok(CliArgs {
        input_file,
        output_file,
        selections,
    })
}

// The main function now correctly handles the `Result` from `parse_arguments`.
fn main() {
    // Call the parsing function and handle its result.
    match parse_arguments() {
        Ok(args) => {
            // If parsing was successful, proceed with the application logic.
            process_files(&args);
        }
        Err(e) => {
            // If parsing failed, print the error and exit gracefully.
            e.exit();
        }
    }
}

// This function contains the core application logic and is completely
// decoupled from the command-line parsing details.
fn process_files(args: &CliArgs) {
    println!("Processing input file: {}", args.input_file);
    println!("Writing to output file: {}", args.output_file);

    println!("Selected items:");
    for selection in &args.selections {
        println!("- {}", selection);
    }

    // Add your file processing logic here.
}
