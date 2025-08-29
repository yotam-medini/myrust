use clap::{Arg, Command, Error};

// The CliArgs struct holds the parsed and validated command-line arguments.
// This provides a clean interface for the main application logic.
struct CliArgs {
    input_file: String,
    output_file: String,
    selections: Vec<String>,
}

// A custom validation and parsing function for the `selection` argument.
// It checks if the input string contains only alphanumeric characters.
// It now returns a `Result<String, String>`, which correctly tells clap
// that the argument's value should be a String.
fn is_valid_selection(val: &str) -> Result<String, String> {
    if val.chars().all(|c| c.is_alphanumeric()) {
        Ok(val.to_string())
    } else {
        Err(String::from("Selection must contain only alphanumeric characters"))
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
