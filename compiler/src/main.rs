// This file is part of Sillyverse.
// Copyright (C) 2017, 2018, Aidin Gharibnavaz <aidin@aidinhut.com>
//
// Sillyverse is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Sillyverse is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Sillyverse. If not, see <http://www.gnu.org/licenses/>.

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

fn compile_file(input_path: &String, output_path: &String) {
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

        let instruction_bytes = [((instruction & 0b1111111100000000u16) >> 8) as u8,
                                         instruction as u8];
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

    compile_file(&input_file, &output_file);
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::env::temp_dir;
    use std::io::Read;

    #[test]
    fn application_1() {
        let mut assembly_file = temp_dir();
        assembly_file.push("test_application_1_df3457392");

        let mut f = File::create(&assembly_file).unwrap();

        f.write_all(b"; A meaningless program.\n\
                    COPY   M1 R2\n \
                    COPY   M7 R1\n \
                    ADD    R0 R1\n \
                    COPY   R1 RPM5\n \
                    JUMP    R2 ;zero\n \
                    NOP\n \
                    NOP\n \
                    NOP\n").unwrap();

        f.flush().unwrap();

        let input_path = String::from(assembly_file.to_str().unwrap());
        let output_path = format!("{}.bin", input_path);

        compile_file(&input_path, &output_path);

        let mut output_file = File::open(output_path).unwrap();
        let mut output_content: Vec<u8> = Vec::new();
        output_file.read_to_end(&mut output_content).unwrap();

        let expected_result: Vec<u8> =
            vec![0b0001_0100u8, 0b01_000010u8,
                 0b0001_0101u8, 0b11_000001u8,
                 0b0010_0000u8, 0b00_000001u8,
                 0b0001_0000u8, 0b01_110101u8,
                 0b0000_0000u8, 0b01_000010u8,
                 0u8, 0u8,
                 0u8, 0u8,
                 0u8, 0u8,];

        for i in 0..8 {
            assert_eq!(output_content[i], expected_result[i]);
        }
    }
}
