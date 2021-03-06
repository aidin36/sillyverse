// This file is part of Sillyverse.
// Copyright (C) 2017-2020, Aidin Gharibnavaz <aidin@aidinhut.com>
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

/// Contains functions to translate assembly literals to their equivalent binary instructions.

use std::collections::HashMap;


pub struct Translator {
    operations_map: HashMap<&'static str, fn(Vec<String>) -> Result<u16, String>>,
}

impl Translator {

    pub fn new() -> Translator {
        let mut map: HashMap<&'static str, fn(Vec<String>) -> Result<u16, String>> = HashMap::new();

        map.insert("data", data);
        map.insert("nop", nop);
        map.insert("subroutine", subroutine);
        map.insert("return", return_subroutine);
        map.insert("syscall", syscall);
        map.insert("copy", copy);
        map.insert("jump", jump);
        map.insert("skip_if_zero", skip_if_zero);
        map.insert("add", add);
        map.insert("subtract", subtract);
        map.insert("skip_if_equal", skip_if_equal);
        map.insert("skip_if_greater", skip_if_greater);
        map.insert("set", set);


        Translator {
            operations_map: map,
        }
    }

    /// Translates one single line into its binary representation.
    /// Returns None if this line presents nothing (a comment or empty line).
    pub fn translate_line(&self, line: String) -> Result<Option<u16>, String> {

        let line = line.trim();

        if line.starts_with(";") {
            // A comment line.
            return Ok(None);
        }

        let line_parts = self.extract_parts(line);

        if line_parts.len() == 0 {
            // Nothing was in this line.
            return Ok(None);
        }

        // Finding function to handle this operation.
        let func = match self.operations_map.get(&*line_parts[0]) {
            Some(f) => f,
            None => return Err(format!("Unknown operation: [{}]", line_parts[0])),
        };

        // Executing the func.
        let result = func(line_parts)?;

        return Ok(Some(result));
    }

    /// Splits the line from spaces, and returns a list of line parts. i.e. operation
    /// and its parameters. It converts all words to lower case.
    fn extract_parts(&self, line: &str) -> Vec<String> {
        let line_split = line.split(" ");
        let mut result: Vec<String> = Vec::new();

        for part in line_split {
            if part.starts_with(" ") || part == "" {
                // Two spaces together, or an empty line.
                continue;
            }
            if part.starts_with(";") {
                // There's comment from now on.
                break;
            }

            result.push(part.to_lowercase());
        }

        return result;
    }

}

/// Translates an string to its equivalent 6 bit address.
fn translate_address(address_str: &String) -> Result<u8, String> {
    let address_type: u8;
    let address_value_str: String;

    // Our address are 6 bits. So, first two bits are always zero.
    // Second two bits are address type, and rest of it is the address value.

    // Checking address type.
    if address_str.starts_with("rpm") {
        address_type = 0b00110000u8;
        address_value_str = address_str.replace("rpm", "");

    } else if address_str.starts_with("rp") {
        address_type = 0b00100000u8;
        address_value_str = address_str.replace("rp", "");

    } else if address_str.starts_with("m") {
        address_type = 0b00010000u8;
        address_value_str = address_str.replace("m", "");

    } else if address_str.starts_with("r") {
        address_type = 0b00000000u8;
        address_value_str = address_str.replace("r", "");

    } else {
        return Err(format!("Unknown address type: {}", address_str));
    }

    // Trying to get address value.
    let address_value = match address_value_str.parse::<u8>() {
        Ok(v) => v,
        Err(error) => return Err(format!(
            "[{}] is not a number. Error while parsing: {}",
            address_value_str,
            error)),
    };

    if address_value > 7 {
        return Err(format!("Expected address less than 7, found: {}", address_value));
    }

    // Appending address type and its value.
    return Ok(address_type | address_value);
}


/// DATA means no operation, just a data that will be stored on that block
/// of memory.
fn data(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(format!("DATA requires exactly one argument, {} found.", args.len() - 1));
    }

    let data = match args[1].parse::<u16>() {
        Ok(v) => v,
        Err(error) => return Err(format!(
            "Argument of DATA must be a positive number less than 65536. Argument: [{}] Error: {}",
            args[1], error)),
    };

    return Ok(data);
}

fn nop(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 1 {
        return Err(String::from("NOP doesn't accept arguments."));
    }

    return Ok(0u16);
}

fn subroutine(args: Vec<String>) -> Result<u16, String> {
    if args.len() != 2 {
        return Err(format!("SUBROUTINE requires exactly one arguments, {} given.", args.len() -1));
    }

    let address = translate_address(&args[1])?;

    return Ok(0b0000_000011_000000u16 | (address as u16));
}

fn return_subroutine(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 1 {
        return Err(String::from("RETURN doesn't accept arguments."));
    }

    return Ok(0b0000000000_000010u16);
}

fn syscall(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 1 {
        return Err(String::from("SYSCALL doesn't accept arguments."));
    }

    return Ok(0b0000000000_000001u16);
}

fn copy(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 3 {
        return Err(format!("COPY requires exactly two arguments, {} given.", args.len() -1));
    }

    let first_address = translate_address(&args[1])?;
    let second_address = translate_address(&args[2])?;

    let first_address: u16 = (first_address as u16) <<6;
    return Ok(0b0001_000000000000u16 | first_address | (second_address as u16));
}

fn jump(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(format!("JUMP requires exactly one arguments, {} given.", args.len() -1));
    }

    let address = translate_address(&args[1])?;

    return Ok(0b0000_000001_000000u16 | (address as u16));
}

fn skip_if_zero(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(format!("SKIP_IF_ZERO requires exactly one arguments, {} given.", args.len() -1));
    }

    let address = translate_address(&args[1])?;

    return Ok(0b0000_000010_000000u16 | (address as u16));
}

fn add(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 3 {
        return Err(format!("ADD requires exactly two arguments, {} given.", args.len() -1));
    }

    let first_address = translate_address(&args[1])?;
    let second_address = translate_address(&args[2])?;

    let first_address: u16 = (first_address as u16) <<6;
    return Ok(0b0010_000000000000u16 | first_address | (second_address as u16));
}

fn subtract(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 3 {
        return Err(format!("SUBTRACT requires exactly two arguments, {} given.", args.len() -1));
    }

    let first_address = translate_address(&args[1])?;
    let second_address = translate_address(&args[2])?;

    let first_address: u16 = (first_address as u16) <<6;
    return Ok(0b0011_000000000000u16 | first_address | (second_address as u16));
}

fn skip_if_equal(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 3 {
        return Err(
            format!("SKIP_IF_EQUAL requires exactly two arguments, {} given.", args.len() -1));
    }

    let first_address = translate_address(&args[1])?;
    let second_address = translate_address(&args[2])?;

    let first_address: u16 = (first_address as u16) <<6;
    return Ok(0b00101_000000000000u16 | first_address | (second_address as u16));
}

fn skip_if_greater(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 3 {
        return Err(
            format!("SKIP_IF_GREATER requires exactly two arguments, {} given.", args.len() -1));
    }

    let first_address = translate_address(&args[1])?;
    let second_address = translate_address(&args[2])?;

    let first_address: u16 = (first_address as u16) <<6;
    return Ok(0b00110_000000000000u16 | first_address | (second_address as u16));
}

fn set(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 3 {
        return Err(format!("SET requires exactly two arguments, {} given.", args.len() -1));
    }

    if !args[1].starts_with("r") {
        return Err(format!("SET only accepts register addresses. Found: {}", args[1]));
    }

    let register_number = match args[1].replace("r", "").parse::<u8>() {
        Ok(v) => v,
        Err(e) =>
            return Err(format!("Provided register is not a number: [{}]. Error: {}", args[1], e)),
    };

    if register_number > 7 {
        return Err(format!("Register number should be less than 7: {}", args[1]));
    }

    let constant = match args[2].parse::<u16>() {
        Ok(v) => v,
        Err(e) => return Err(format!(
            "Second argument of SET must be a positive number: [{}] Error: {}", args[2], e)),
    };

    if constant >= 512 {
        return Err(format!("Constant of SET should be less than 512: [{}]", constant));
    }

    return Ok(0b0110_000_000000000u16 | ((register_number as u16) <<9) | constant);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn bad_line() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("BAD_COMMAND  12"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("  3000 DATA "));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn comment() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("; comment")).unwrap();
        assert_eq!(result.is_none(), true);

        let result = translator.translate_line(String::from("    ; comment")).unwrap();
        assert_eq!(result.is_none(), true);
    }

    #[test]
    fn data() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("DATA     3000")).unwrap();
        assert_eq!(result.unwrap(), 3000);

        let result = translator.translate_line(String::from(" Data  65535  ")).unwrap();
        assert_eq!(result.unwrap(), 65535);

        // Errors
        let result = translator.translate_line(String::from(" DATA  "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from(" DATA  120  200"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from(" DATA  0xFF "));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn nop() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("NOP")).unwrap();
        assert_eq!(result.unwrap(), 0u16);

        let result = translator.translate_line(String::from("nop  ")).unwrap();
        assert_eq!(result.unwrap(), 0u16);

        let result = translator.translate_line(String::from("nOp ; Comment")).unwrap();
        assert_eq!(result.unwrap(), 0u16);

        let result = translator.translate_line(String::from("NOP  R1"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn subroutine() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("SubRoutine R2 ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000011_000010u16);

        let result = translator.translate_line(String::from("SUBROUTIne  m5 ;comment ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000011_010101u16);

        let result = translator.translate_line(String::from("subroutine Rp5")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000011_100101u16);

        let result = translator.translate_line(String::from("SUBroutine RPm5")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000011_110101u16);

        // Testing errors.

        let result = translator.translate_line(String::from("SUBROUTINE "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("Subroutine ; comment"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SUBROUTINE R1 R4"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SUBROUTINE 14"));
        assert_eq!(result.is_err(), true);

    }

    #[test]
    fn return_subroutine() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("RETURN")).unwrap();
        assert_eq!(result.unwrap(), 2u16);

        let result = translator.translate_line(String::from("ReTuRn  ")).unwrap();
        assert_eq!(result.unwrap(), 2u16);

        let result = translator.translate_line(String::from("return ; A ;Comment")).unwrap();
        assert_eq!(result.unwrap(), 2u16);

        let result = translator.translate_line(String::from("RETURN  R1"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn syscall() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("  SYSCALL  ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000000000_000001u16);

        let result = translator.translate_line(String::from("SysCall ; A comment NOP ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000000000_000001u16);

        let result = translator.translate_line(String::from("SYSCALL  R1"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn copy() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("COPY R1 M6")).unwrap();
        assert_eq!(result.unwrap(), 0b0001_000001_010110u16);

        let result = translator.translate_line(String::from("COPY   RP2  RPM3")).unwrap();
        assert_eq!(result.unwrap(), 0b0001_100010_110011u16);

        // Testing errors.

        let result = translator.translate_line(String::from("COPY  M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("COPY M20 M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("COPY ;bad copy"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("COPY 120 14"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn jump() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("JUMP R1  ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000001_000001u16);

        let result = translator.translate_line(String::from("jump  m3 ;comment R2 ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000001_010011u16);

        let result = translator.translate_line(String::from("JuMp Rp4")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000001_100100u16);

        let result = translator.translate_line(String::from("JUmP RPm5")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000001_110101u16);

        // Testing errors.

        let result = translator.translate_line(String::from("JUMP "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("JUMP ; comment"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("JUMP R1 R4"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("JUMP 14"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn skip_if_zero() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("SKIP_IF_ZERO R1  ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000010_000001u16);

        let result = translator.translate_line(String::from("skip_if_zero  m7 ;comment R2 ")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000010_010111u16);

        let result = translator.translate_line(String::from("skip_IF_zero Rp0")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000010_100000u16);

        let result = translator.translate_line(String::from("SKIP_IF_ZERO RPm5")).unwrap();
        assert_eq!(result.unwrap(), 0b0000_000010_110101u16);

        // Testing errors.

        let result = translator.translate_line(String::from("SKIP_IF_ZERO "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_ZERO ; comment"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_ZERO M1 RP4"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_ZERO R12"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_ZERO 0"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn add() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("ADD R1 M6")).unwrap();
        assert_eq!(result.unwrap(), 0b0010_000001_010110u16);

        let result = translator.translate_line(String::from("ADD   RP2  RPM3")).unwrap();
        assert_eq!(result.unwrap(), 0b0010_100010_110011u16);

        // Testing errors.

        let result = translator.translate_line(String::from("ADD  M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("ADD M20 M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("ADD 120 14"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("ADD"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn subtract() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("SUBTRACT R3 M5")).unwrap();
        assert_eq!(result.unwrap(), 0b0011_000011_010101u16);

        let result = translator.translate_line(String::from("subtract   RP7  rpm3")).unwrap();
        assert_eq!(result.unwrap(), 0b0011_100111_110011u16);

        // Testing errors.

        let result = translator.translate_line(String::from("SUBTRACT  M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SUBTRACT M20 M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SUBTRACT 120 14"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SUBTRACT"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn skip_if_equal() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("SKIP_IF_EQUAL R3 M6")).unwrap();
        assert_eq!(result.unwrap(), 0b0101_000011_010110u16);

        let result = translator.translate_line(String::from("skip_if_equal   m2  RPM3")).unwrap();
        assert_eq!(result.unwrap(), 0b0101_010010_110011u16);

        // Testing errors.

        let result = translator.translate_line(String::from("SKIP_IF_EQUAL  M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_EQUAL M2 RPM8"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_EQUAL 120 14"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_EQUAL"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn skip_if_greater() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("   SKIP_IF_GREATER  R3 M6 ;M80")).unwrap();
        assert_eq!(result.unwrap(), 0b0110_000011_010110u16);

        let result = translator.translate_line(String::from("skip_if_greater   M0  RPM3")).unwrap();
        assert_eq!(result.unwrap(), 0b0110_010000_110011u16);

        // Testing errors.

        let result = translator.translate_line(String::from("SKIP_IF_GREATER  M2"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_GREATER M2 R9"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_GREATER 120 14"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SKIP_IF_GREATER"));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn set() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("SET R1 120")).unwrap();
        assert_eq!(result.unwrap(), 0b0110_001_001111000u16);

        let result = translator.translate_line(String::from("  set r7 511  ;")).unwrap();
        assert_eq!(result.unwrap(), 0b0110_111_111111111u16);

        let result = translator.translate_line(String::from("  SET   R0 0 ")).unwrap();
        assert_eq!(result.unwrap(), 0b0110_000_000000000u16);

        // Checking errors.

        let result = translator.translate_line(String::from("  SET   R0 "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SET   R9 10 "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SET  R1 10 32 "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SET R0 512 "));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SET R1 R12"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SET R1 R7"));
        assert_eq!(result.is_err(), true);

        let result = translator.translate_line(String::from("SET M1 20"));
        assert_eq!(result.is_err(), true);

    }

}
