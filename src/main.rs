mod embl_2_json;
mod json_2_embl;

fn main() {
    //get the command line arguments
    let args: Vec<String> = std::env::args().collect();

    //check if the number of arguments is correct
    if args.len() < 2 {
        //print the usage and exit the program
        eprintln!("Usage: <commnad> <input.embl> <output.json>");
        std::process::exit(1);
    }

    //get the command
    let command = args[1].clone();

    //match on every possible command
    match command.as_str() {
        "process" => {
            //check if the number of arguments is correct
            if args.len() < 4 {
                //print the usage and exit the program
                eprintln!("Usage: processs <input.embl> <output.json>");
                std::process::exit(1);
            }
            //get the input from the arguments
            let input_embl = &args[2];
            //get the output from the arguments
            let output_json = &args[3];
            //call the process_embl function from the embl_2_json module
            embl_2_json::process_embl(input_embl, output_json);
        }

        "convert" => {
            //check if the number of arguments is correct
            if args.len() < 4 {
                //print the usage and exit the program
                eprintln!("Usage: convert <input.json> <output.embl>");
                std::process::exit(1);
            }
            //get the input from the arguments
            let input_json = &args[2];
            //get the output from the arguments
            let output_embl = &args[3];
            let _ = json_2_embl::convert_json(input_json, output_embl);
        }

        _ => {
            //print the usage and exit the program
            eprintln!("Usage: <commnad> <input.embl> <output.json>");
            std::process::exit(1);
        }
    }
}

