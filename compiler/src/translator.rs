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

        println!("{:b}", result);

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
    let mut address_type = 0u8;
    let mut address_value_str: String = String::new();

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
        return Err(String::from("COPY requires exactly two arguments."));
    }

    let first_address = translate_address(&args[1])?;
    let second_address = translate_address(&args[2])?;

    let first_address: u16 = (first_address as u16) <<6;
    return Ok(0b0001_000000000000u16 | first_address | (second_address as u16));
}
