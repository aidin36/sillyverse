/// Contains functions to translate assembly literals to their equivalent binary instructions.

use std::collections::HashMap;


pub struct Translator {
    operations_map: HashMap<&'static str, fn(Vec<String>) -> Result<u16, String>>,
}

impl Translator {

    pub fn new() -> Translator {
        let mut map: HashMap<&'static str, fn(Vec<String>) -> Result<u16, String>> = HashMap::new();

        map.insert("nop", nop);
        map.insert("copy", copy);
        map.insert("jump", jump);
        map.insert("skip_if_zero", skip_if_zero);
        map.insert("add", add);
        map.insert("subtract", subtract);
        map.insert("skip_if_equal", skip_if_equal);
        map.insert("skip_if_greater", skip_if_greater);

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

fn nop(args: Vec<String>) -> Result<u16, String> {

    if args.len() != 1 {
        return Err(String::from("NOP doesn't accept arguments."));
    }

    return Ok(0u16);
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn comment() {
        let translator = Translator::new();

        let result = translator.translate_line(String::from("; comment")).unwrap();
        assert_eq!(result.is_none(), true);

        let result = translator.translate_line(String::from("    ; comment")).unwrap();
        assert_eq!(result.is_none(), true);
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

        let result = translator.translate_line(String::from("subtract   RP7  RPM3")).unwrap();
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

        let result = translator.translate_line(String::from("skip_if_equal   M2  RPM3")).unwrap();
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
}
