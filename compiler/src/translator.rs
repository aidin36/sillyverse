/// Contains functions to translate assembly literals to their equivalent binary instructions.

use std::collections::HashMap;


pub struct Translator {
    operations_map: HashMap<&'static str, fn(Vec<String>) -> Result<u16, String>>,
}

impl Translator {

    pub fn new() -> Translator {
        let mut map: HashMap<&'static str, fn(Vec<String>) -> Result<u16, String>> = HashMap::new();

        map.insert("nop", nop);

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

fn nop(params: Vec<String>) -> Result<u16, String> {

    if params.len() != 1 {
        return Err(String::from("NOP doesn't accept parameters."));
    }

    return Ok(0u16);
}
