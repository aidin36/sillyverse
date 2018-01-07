/// This module defines micro-operations of the CPU.

use std::collections::HashMap;
use hardware::Hardware;
use hardware::operation_code::OperationCode;


pub struct Operations {
    functions: HashMap<OperationCode, fn(&mut Hardware, u16) -> Result<(), String>>,
}

impl Operations {
    pub fn new() -> Operations {
        let mut map: HashMap<OperationCode, fn(&mut Hardware, u16) -> Result<(), String>> =
            HashMap::new();

        map.insert(OperationCode::new(0b0000000000000000u16), nop);

        Operations {
            functions: map,
        }
    }

    pub fn get_function(&self, instruction: u16) -> Result<fn(&mut Hardware, u16) -> Result<(), String>, String> {
        match self.functions.get(&OperationCode::new(instruction)) {
            Some(&function) => return Ok(function),
            None => return Err(format!("Unknown instruction: [{:b}]", instruction)),
        }
    }
}

/// Gets operand, from the specified address.
fn get_operand(hardware: &mut Hardware, address: u8) -> Result<u16, String> {

    // Out addresses is 6 bit, so the first two bits are ignored.
    // Second two bits shows address type, and the rest (4 bits)
    // is the value that will be interpreted according to the type.

    let address_type = address & 0b00_11_0000u8;
    let address_value = address & 0b0000_1111u8;

    if address_value > 7 {
        return Err(format!("Invalid register number. [{}]", address_value));
    }

    let address_value = address_value as usize;

    if address_type == 0b00_00_0000u8 {
        // Register is operand.
        return Ok(hardware.registers[address_value]);

    } else if address_type == 0b00_01_0000u8 {
        // Register is address of operand.
        let operand_address = hardware.registers[address_value] as usize;

        if operand_address >= hardware.memory.len() {
            return Err(format!(
                "Address is out of memory. Address was [{}] stored in register [{}].",
                operand_address, address_value));
        }

        return Ok(hardware.memory[operand_address]);

    } else if address_type == 0b00_10_0000u8 {
        // Register + Program Counter is the operand.
        // We used "saturating_add" so result will be max of u16 if value becomes
        // too large.
        return Ok(hardware.registers[address_value].saturating_add(hardware.program_counter));

    } else if address_type == 0b00_11_0000u8 {
        // Register + Program Counter is the address of operand.
        let (operand_address, is_overflowed) =
            hardware.registers[address_value].overflowing_add(hardware.program_counter);
        let operand_address = operand_address as usize;

        if is_overflowed {
            return Err(format!(
                "Memory address overflow. PC ({}) + Register{} ({})",
                hardware.program_counter, address_value, hardware.registers[address_value]))
        }

        if operand_address >= hardware.memory.len() {
            return Err(format!(
                "Address is out of memory. Address was [{}] stored in register [{}].",
                operand_address, address_value));
        }

        return Ok(hardware.memory[operand_address]);
    }

    // We checked all of possibilities. We should never reach here.
    return Err(format!(
        "Unknown memory address type. Address was: [{:b}] This is a bug! Please report it.",
        address));
}

fn nop(hardware: &mut Hardware, _instruction: u16) -> Result<(), String> {
    hardware.program_counter += 1;
    return Ok(());
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Tests address type one.
    #[test]
    fn get_operand_type_one() {

        let mut hardware = Hardware::new(1);

        hardware.registers[0] = 128;
        hardware.registers[5] = 254;
        hardware.registers[7] = 1;

        let operand = get_operand(&mut hardware,0b0000_0000u8).unwrap();
        assert_eq!(operand, 128);

        let operand = get_operand(&mut hardware,0b0000_0101u8).unwrap();
        assert_eq!(operand, 254);

        let operand = get_operand(&mut hardware,0b0000_0111u8).unwrap();
        assert_eq!(operand, 1);

        // Bad register address.
        let operand = get_operand(&mut hardware,0b0000_1000u8);
        assert_eq!(operand.is_err(), true);
    }

    /// Tests address type one.
    #[test]
    fn get_operand_type_two() {

        let mut hardware = Hardware::new(10);

        hardware.registers[0] = 0;
        hardware.registers[5] = 3;
        hardware.registers[7] = 9;

        hardware.memory[0] = 128;
        hardware.memory[3] = 64;
        hardware.memory[9] = 1;

        let operand = get_operand(&mut hardware,0b0001_0000u8).unwrap();
        assert_eq!(operand, 128);
        let operand = get_operand(&mut hardware,0b0001_0101u8).unwrap();
        assert_eq!(operand, 64);
        let operand = get_operand(&mut hardware,0b0001_0111u8).unwrap();
        assert_eq!(operand, 1);

        // Bad address.
        hardware.registers[1] = 14;
        let operand = get_operand(&mut hardware,0b0001_0001u8);
        assert_eq!(operand.is_err(), true);
    }

    /// Tests address type one.
    #[test]
    fn get_operand_type_three() {

        let mut hardware = Hardware::new(1);

        hardware.program_counter = 9;
        hardware.registers[2] = 7;
        let operand = get_operand(&mut hardware,0b0010_0010u8).unwrap();
        assert_eq!(operand, 16);

        hardware.program_counter = 0;
        hardware.registers[1] = 18;
        let operand = get_operand(&mut hardware,0b0010_0001u8).unwrap();
        assert_eq!(operand, 18);

        hardware.program_counter = 12;
        hardware.registers[3] = 0;
        let operand = get_operand(&mut hardware,0b0010_0011u8).unwrap();
        assert_eq!(operand, 12);

        // Saturating add
        hardware.program_counter = 65000;
        hardware.registers[6] = 17000;
        let operand = get_operand(&mut hardware,0b0010_0110u8).unwrap();
        assert_eq!(operand, 65535u16);
    }

    /// Tests address type one.
    #[test]
    fn get_operand_type_four() {

        let mut hardware = Hardware::new(20);

        hardware.program_counter = 9;
        hardware.registers[2] = 7;
        hardware.memory[16] = 120;
        let operand = get_operand(&mut hardware,0b0011_0010u8).unwrap();
        assert_eq!(operand, 120);

        hardware.program_counter = 0;
        hardware.registers[1] = 18;
        hardware.memory[18] = 254;
        let operand = get_operand(&mut hardware,0b0011_0001u8).unwrap();
        assert_eq!(operand, 254);

        hardware.program_counter = 12;
        hardware.registers[3] = 0;
        hardware.memory[12] = 107;
        let operand = get_operand(&mut hardware,0b0011_0011u8).unwrap();
        assert_eq!(operand, 107);

        // Overflow
        hardware.program_counter = 65000;
        hardware.registers[6] = 17000;
        let operand = get_operand(&mut hardware,0b0011_0110u8);
        assert_eq!(operand.is_err(), true);

        // Out of memory.
        hardware.program_counter = 11;
        hardware.registers[0] = 10;
        let operand = get_operand(&mut hardware,0b0011_0000u8);
        assert_eq!(operand.is_err(), true);
    }
}