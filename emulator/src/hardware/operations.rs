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

/// This module defines micro-operations of the CPU.

use std::collections::HashMap;
use hardware::Hardware;
use hardware::operation_code::OperationCode;
use CPUState;


pub struct Operations {
    functions: HashMap<OperationCode, fn(&mut Hardware, u16) -> Result<(), String>>,
}

impl Operations {
    pub fn new() -> Operations {
        let mut map: HashMap<OperationCode, fn(&mut Hardware, u16) -> Result<(), String>> =
            HashMap::new();

        // No operand operations
        map.insert(OperationCode::new(0b0000000000_000000u16), nop);
        map.insert(OperationCode::new(0b0000000000_000001u16), syscall);
        map.insert(OperationCode::new(0b0000000000_000010u16), return_subroutine);

        // Single operand operations
        map.insert(OperationCode::new(0b0000_000001_000000u16), jump);
        map.insert(OperationCode::new(0b0000_000010_000000u16), skip_if_zero);
        map.insert(OperationCode::new(0b0000_000011_000000u16), subroutine);

        // Double operand operations
        map.insert(OperationCode::new(0b0001_000000000000u16), copy);
        map.insert(OperationCode::new(0b0010_000000000000u16), add);
        map.insert(OperationCode::new(0b0011_000000000000u16), subtract);
        map.insert(OperationCode::new(0b0100_000000000000u16), skip_if_equal);
        map.insert(OperationCode::new(0b0101_000000000000u16), skip_if_greater);
        map.insert(OperationCode::new(0b0110_000000000000u16), set);

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
    let first_address = ((instruction & 0b0000_111111_000000u16) >> 6) as u8;
    let second_address = (instruction & 0b0000_000000_111111u16) as u8;

    return (first_address, second_address);
}

/// Extracts value that an address is pointing to, from a
/// one-operand instruction.
fn extract_one_operand_value(hardware: &Hardware, instruction: u16, supports_register_pc: bool)
    -> Result<u16, String> {

    let address = extract_one_operand_address(instruction);

    let true_address = get_true_address(hardware, address)?;

    let value = match true_address {
        Address::Register(register_number) =>
            hardware.registers[register_number as usize],
        Address::Memory(memory_address) =>
            hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(jump_address) => {
            if !supports_register_pc {
                return Err(format!("Unsupported address type. Instruction: {:b}", instruction));
            }
            jump_address
        },
    };

    return Ok(value);
}

/// Extracts value that an address is pointing to, from a
/// two-operand instruction.
///
/// @supports_register_pc: Whether the operation supports RegisterPlusPC address type.
///     If set to false, an Err will return in case of RegisterPlusPC address.
fn extract_two_operand_value(hardware: &Hardware, instruction: u16, supports_register_pc: bool)
    -> Result<(u16, u16), String> {

    let (first_address, second_address) = extract_two_operand_address(instruction);

    let first_true_address = get_true_address(hardware, first_address)?;
    let first_value = match first_true_address {
        Address::Register(register_number) =>
            hardware.registers[register_number as usize],
        Address::Memory(memory_address) =>
            hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(jump_address) => {
            if !supports_register_pc {
                return Err(format!("Unsupported address type. Instruction: {:b}", instruction));
            }
            jump_address
        },
    };

    let second_true_address = get_true_address(hardware, second_address)?;
    let second_value = match second_true_address {
        Address::Register(register_number) =>
            hardware.registers[register_number as usize],
        Address::Memory(memory_address) =>
            hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(jump_address) => {
            if !supports_register_pc {
                return Err(format!("Unsupported address type. Instruction: {:b}", instruction));
            }
            jump_address
        },
    };

    return Ok((first_value, second_value));
}

/// It just increases program counter (skips this instruction).
fn nop(hardware: &mut Hardware, _instruction: u16) -> Result<(), String> {
    hardware.program_counter += 1;
    return Ok(());
}

/// Do a sys call. Each sys call has its own conventions. See documentation.
fn syscall(hardware: &mut Hardware, _instruction: u16) -> Result<(), String> {

    let mut cpu_state = CPUState::new(&hardware.registers);

    // Calling the sys call.
    hardware.call_syscall(&mut cpu_state)?;

    // Setting changed registers in the hardware.
    for i in 0..hardware.registers.len() {
        hardware.registers[i] = cpu_state.get_register(i);
    }

    hardware.program_counter += 1;

    // Checking for errors.
    if cpu_state.get_error_flag() {
        hardware.error_flag = true;
        return Err(String::from("Something went wrong when sys call is called."));
    }

    return Ok(());
}

fn return_subroutine(hardware: &mut Hardware, _instruction: u16) -> Result<(), String> {

   match hardware.call_stack.pop() {
        Some(pc) =>  hardware.program_counter = pc,
        None => {
            hardware.underflow_flag = true;
            return Err(String::from("Call stack underflow"));
        }
    };

    return Ok(());
}

/// Jumps to the address inside the instruction.
fn jump(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {

    hardware.program_counter =
        extract_one_operand_value(hardware, instruction, true)?;

    return Ok(());
}

fn subroutine(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {

    if hardware.call_stack.len() == Hardware::get_call_stack_size() {
        hardware.overflow_flag = true;
        return Err(String::from("Call stack overflow."));
    }

    // Storing return address.
    hardware.call_stack.push(hardware.program_counter + 1);

    // Jumping.
    hardware.program_counter =
        extract_one_operand_value(hardware, instruction, true)?;

    return Ok(());
}

/// Skips next instruction if operand is pointing to an address with zero value.
fn skip_if_zero(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {

    let address_value = extract_one_operand_value(hardware, instruction, false)?;

    if address_value == 0 {
        hardware.program_counter += 2;
    } else {
        hardware.program_counter += 1;
    }

    return Ok(());
}

/// Copy value of an address to another.
fn copy(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {
    let (source_address, destination_address) = extract_two_operand_address(instruction);

    let source_true_address = get_true_address(hardware, source_address)?;
    let source_value = match source_true_address {
        Address::Register(register_number) => hardware.registers[register_number as usize],
        Address::Memory(memory_address) => hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for COPY. Instruction: {:b}",
                               instruction)),
    };

    let destination_true_address = get_true_address(hardware, destination_address)?;
    match destination_true_address {
        Address::Register(register_number) =>
            hardware.registers[register_number as usize] = source_value,
        Address::Memory(memory_address) =>
            hardware.memory[memory_address as usize] = source_value,
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid destination address type for COPY. Instruction: {:b}",
                               instruction)),
    }

    hardware.program_counter += 1;

    return Ok(());
}

/// Adds two values.
fn add(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {
    let (first_address, second_address) = extract_two_operand_address(instruction);

    let true_first_address = get_true_address(hardware, first_address)?;
    let first_value = match true_first_address {
        Address::Register(register_number) => hardware.registers[register_number as usize],
        Address::Memory(memory_address) => hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for ADD. Instruction: {:b}",
                               instruction)),
    };

    let true_second_address = get_true_address(hardware, second_address)?;
    let second_value = match true_second_address {
        Address::Register(register_number) => hardware.registers[register_number as usize],
        Address::Memory(memory_address) => hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for ADD. Instruction: {:b}",
                               instruction)),
    };

    let result = first_value.saturating_add(second_value);

    // Storing the result back to the second address.
    match true_second_address {
        Address::Register(register_number) => hardware.registers[register_number as usize] = result,
        Address::Memory(memory_address) => hardware.memory[memory_address as usize] = result,
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for ADD. Instruction: {:b}",
                               instruction)),
    }

    hardware.program_counter += 1;

    return Ok(());
}

/// Subtracts two values.
fn subtract(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {
    let (first_address, second_address) = extract_two_operand_address(instruction);

    let true_first_address = get_true_address(hardware, first_address)?;
    let first_value = match true_first_address {
        Address::Register(register_number) => hardware.registers[register_number as usize],
        Address::Memory(memory_address) => hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for SUBTRACT. Instruction: {:b}",
                               instruction)),
    };

    let true_second_address = get_true_address(hardware, second_address)?;
    let second_value = match true_second_address {
        Address::Register(register_number) => hardware.registers[register_number as usize],
        Address::Memory(memory_address) => hardware.memory[memory_address as usize],
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for SUBTRACT. Instruction: {:b}",
                               instruction)),
    };

    let result = first_value.saturating_sub(second_value);

    // Storing the result back to the second address.
    match true_second_address {
        Address::Register(register_number) => hardware.registers[register_number as usize] = result,
        Address::Memory(memory_address) => hardware.memory[memory_address as usize] = result,
        Address::RegisterPlusPC(_) =>
            return Err(format!("Invalid source address type for SUBTRACT. Instruction: {:b}",
                               instruction)),
    }

    hardware.program_counter += 1;

    return Ok(());
}

/// Skips the next instruction if value of two operands are equal.
fn skip_if_equal(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {

    let (first_value, second_value) =
        extract_two_operand_value(hardware, instruction, false)?;

    if first_value == second_value {
        hardware.program_counter += 2;
    } else {
        hardware.program_counter += 1;
    }

    return Ok(());
}

/// Skips the next instruction if value of first operand is greater than the second one.
fn skip_if_greater(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {
    let (first_value, second_value) =
        extract_two_operand_value(hardware, instruction, false)?;

    if first_value > second_value {
        hardware.program_counter += 2;
    } else {
        hardware.program_counter += 1;
    }

    return Ok(());
}

/// Sets a constant to a register.
fn set(hardware: &mut Hardware, instruction: u16) -> Result<(), String> {

    let register_number = (0b0000_111_000000000u16 & instruction) >> 9;
    let constant = 0b0000_000_111111111u16 & instruction;

    hardware.registers[register_number as usize] = constant;
    hardware.program_counter += 1;

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests address type one.
    #[test]
    fn get_memory_type_one() {

        let hardware = Hardware::new(1);

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
