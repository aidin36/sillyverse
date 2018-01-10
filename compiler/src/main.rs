use std::env;
use std::process;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::fs::File;
use std::io::Write;

mod translator;


fn print_usage(program_name: String) {
    println!(" ");
    println!("Usage: {} input-file", program_name);
    println!(" ");
}

fn compile_file(input_path: String, output_path: String) {
    let input_file = File::open(input_path).expect("Could not open input file.");
    let output_file = File::create(output_path).expect("Could not open output file.");

    let input_file_reader = BufReader::new(&input_file);
    let mut output_file_writer = BufWriter::new(&output_file);

    let translator = translator::Translator::new();

    for (line_num, line) in input_file_reader.lines().enumerate() {

        if line.is_err() {
            eprintln!("Could not read line from the input file. Line: {}", line_num);
            eprintln!("{}", line.unwrap_err());
            process::exit(2);
        }

        let line_content = line.unwrap();

        let instruction = match translator.translate_line(line_content) {
            Ok(option) => match option {
                None => continue,
                Some(v) => v,
            },
            Err(error) => {
                eprintln!("Compile failed at line: {}", line_num);
                eprintln!("{}", error);
                process::exit(3);
            },
        };

        let instruction_bytes = [instruction as u8,
                                         ((instruction & 0b1111111100000000u16) >> 8) as u8];
        output_file_writer.write_all(&instruction_bytes)
            .expect("Could not write to output file.");
    }
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        print_usage(args.nth(0).unwrap());
        process::exit(1);
    }

    let input_file = args.nth(1).unwrap();
    let output_file = format!("{}.bin", input_file);

    compile_file(input_file, output_file);
}
