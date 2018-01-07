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

        // No operand operations
        map.insert(OperationCode::new(0b0000000000_000000u16), nop);

        // Single operand operations
        map.insert(OperationCode::new(0b0000_000001_000000u16), jump);

        // Double operand operations
        map.insert(OperationCode::new(0b0001_000000000000u16), copy);

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

/// Defines types of address that an operation can have.
enum Address {
    // Value is the register number.
    Register(u8),
    // Value is an address of memory.
    Memory(u16),
    // Value is register's value plus Program Counter.
    RegisterPlusPC(u16),
}

/// Returns the real address that specified "address" is pointing to.
/// For example, "address" points to where the real address stored.
/// Addresses are 6 bits, so the first 2 bits will be ignored.
fn get_true_address(hardware: &Hardware, address: u8) -> Result<Address, String> {

    // Out addresses is 6 bit, so the first two bits are ignored.
    // Second two bits shows address type, and the rest (4 bits)
    // is the value that will be interpreted according to the type.

    let address_type = address & 0b00_11_0000u8;
    let register_number = address & 0b0000_1111u8;

    if register_number > 7 {
        return Err(format!("Invalid register number. [{}]", register_number));
    }

    if address_type == 0b00_00_0000u8 {
        // Address is the register itself.
        return Ok(Address::Register(register_number));

    } else if address_type == 0b00_01_0000u8 {
        // Register points to a memory address.
        let memory_address = hardware.registers[register_number as usize];

        if memory_address as usize >= hardware.memory.len() {
            return Err(format!(
                "Address is out of memory. Address was [{}] stored in register [{}].",
                memory_address, register_number));
        }

        return Ok(Address::Memory(memory_address));

    } else if address_type == 0b00_10_0000u8 {
        // Register value + Program Counter is the memory address.
        // We used "saturating_add" so result will be max of u16 if value becomes
        // too large.
        return Ok(Address::RegisterPlusPC(
                    hardware.registers[register_number as usize]
                        .saturating_add(hardware.program_counter)));

    } else { // address_type == 0b00_11_0000u8
        // Register + Program Counter is pointing to where address stored.
        let (memory_address, is_overflowed) =
            hardware.registers[register_number as usize].overflowing_add(hardware.program_counter);

        if is_overflowed {
            return Err(format!(
                "Memory address overflow. PC ({}) + Register{} ({})",
                hardware.program_counter, register_number,
                hardware.registers[register_number as usize]))
        }

        if memory_address as usize >= hardware.memory.len() {
            return Err(format!(
                "Address is out of memory. Address was [{}] stored in register [{}].",
                memory_address, register_number));
        }

        return Ok(Address::Memory(memory_address));
    }
}


/// Extracts address from a one-operand instruction.
fn extract_one_operand_address(instruction: u16) -> u8 {
    return (instruction & 0b0000_000000_111111u16) as u8;
}

/// Extracts addresses from a two-operand instruction.
fn extract_two_operand_address(instruction: u16) -> (u8, u8) {
    let first_address = (instruction & 0b0000_111111_000000u16) as u8;
    let second_address = (instruction & 0b0000_000000_111111u16) as u8;

    return (first_address, second_address);
}

/// It just increases program counter (skips this instruction).
fn nop(hardware: &mut Hardware, _instruction: u16) -> Result<(), String> {
    hardware.program_counter += 1;
    return Ok(());
}

/// Jumps to the address inside the instruction.
fn jump(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {
    let address = extract_one_operand_address(instruction);

    let true_address = get_true_address(hardware, address)?;

    match true_address {
        Address::Register(register_number) =>
            hardware.program_counter = hardware.registers[register_number as usize],
        Address::Memory(memory_address) =>
            hardware.program_counter = hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(jump_address) =>
            hardware.program_counter = jump_address,
    }

    return Ok(());
}

/// Copy value of an address to another.
fn copy(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {
    let (first_address, second_address) = extract_two_operand_address((instruction));

    return Err(String::from("Not implemented."));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests address type one.
    #[test]
    fn get_memory_type_one() {

        let mut hardware = Hardware::new(1);

        let address = get_true_address(&hardware, 0b0000_0000u8).unwrap();
        match address {
            Address::Register(register_number) => assert_eq!(register_number, 0),
            _ => assert!(false),
        }

        let address = get_true_address(&hardware, 0b0000_0101u8).unwrap();
        match address {
            Address::Register(register_number) => assert_eq!(register_number, 5),
            _ => assert!(false),
        }

        let address = get_true_address(&hardware, 0b0000_0111u8).unwrap();
        match address {
            Address::Register(register_number) => assert_eq!(register_number, 7),
            _ => assert!(false),
        }

        // Bad register address.
        let address = get_true_address(&hardware, 0b0000_1000u8);
        assert_eq!(address.is_err(), true);
    }

    /// Tests address type one.
    #[test]
    fn get_address_type_two() {

        let mut hardware = Hardware::new(10);

        hardware.registers[0] = 0;
        hardware.registers[5] = 3;
        hardware.registers[7] = 9;

        let address = get_true_address(&mut hardware,0b0001_0000u8).unwrap();
        match address {
            Address::Memory(memory_address) => assert_eq!(memory_address, 0),
            _ => assert!(false),
        }

        let address = get_true_address(&mut hardware,0b0001_0101u8).unwrap();
        match address {
            Address::Memory(memory_address) => assert_eq!(memory_address, 3),
            _ => assert!(false),
        }

        let address = get_true_address(&mut hardware,0b0001_0111u8).unwrap();
        match address {
            Address::Memory(memory_address) => assert_eq!(memory_address, 9),
            _ => assert!(false),
        }

        // Bad address.
        hardware.registers[1] = 14;
        let address = get_true_address(&mut hardware,0b0001_0001u8);
        assert_eq!(address.is_err(), true);
    }

    /// Tests address type one.
    #[test]
    fn get_address_type_three() {

        let mut hardware = Hardware::new(1);

        hardware.program_counter = 9;
        hardware.registers[2] = 7;
        let address = get_true_address(&mut hardware,0b0010_0010u8).unwrap();
        match address {
            Address::RegisterPlusPC(memory_address) => assert_eq!(memory_address, 16),
            _ => assert!(false),
        }

        hardware.program_counter = 0;
        hardware.registers[1] = 18;
        let address = get_true_address(&mut hardware,0b0010_0001u8).unwrap();
        match address {
            Address::RegisterPlusPC(memory_address) => assert_eq!(memory_address, 18),
            _ => assert!(false),
        }

        hardware.program_counter = 12;
        hardware.registers[3] = 0;
        let address = get_true_address(&mut hardware,0b0010_0011u8).unwrap();
        match address {
            Address::RegisterPlusPC(memory_address) => assert_eq!(memory_address, 12),
            _ => assert!(false),
        }

        // Saturating add
        hardware.program_counter = 65000;
        hardware.registers[6] = 17000;
        let address = get_true_address(&mut hardware,0b0010_0110u8).unwrap();
        match address {
            Address::RegisterPlusPC(memory_address) => assert_eq!(memory_address, 65535u16),
            _ => assert!(false),
        }
    }

    /// Tests address type one.
    #[test]
    fn get_address_type_four() {

        let mut hardware = Hardware::new(20);

        hardware.program_counter = 9;
        hardware.registers[2] = 7;
        let address = get_true_address(&mut hardware,0b0011_0010u8).unwrap();
        match address {
            Address::Memory(memory_address) => assert_eq!(memory_address, 16),
            _ => assert!(false),
        }

        hardware.program_counter = 0;
        hardware.registers[1] = 18;
        let address = get_true_address(&mut hardware,0b0011_0001u8).unwrap();
        match address {
            Address::Memory(memory_address) => assert_eq!(memory_address, 18),
            _ => assert!(false),
        }

        hardware.program_counter = 12;
        hardware.registers[3] = 0;
        let address = get_true_address(&mut hardware,0b0011_0011u8).unwrap();
        match address {
            Address::Memory(memory_address) => assert_eq!(memory_address, 12),
            _ => assert!(false),
        }

        // Overflow
        hardware.program_counter = 65000;
        hardware.registers[6] = 17000;
        let address = get_true_address(&mut hardware,0b0011_0110u8);
        assert_eq!(address.is_err(), true);

        // Out of memory.
        hardware.program_counter = 11;
        hardware.registers[0] = 10;
        let address = get_true_address(&mut hardware,0b0011_0000u8);
        assert_eq!(address.is_err(), true);
    }
}