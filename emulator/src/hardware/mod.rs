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

/// This module implements the hardware of the emulator.
/// It emulates CPU and memory of a single machine.

mod operations;
mod operation_code;

use std::rc::Weak;
use std::sync::Mutex;
use CPUState;
use SysCallback;


pub struct Hardware {
    memory: Vec<u16>,

    program_counter: u16,
    stack_pointer: u8,

    // There are 8 registers.
    registers: [u16; 8],

    overflow_flag: bool,
    error_flag: bool,

    sys_callback: Option<Weak<Mutex<SysCallback>>>,

    operations: operations::Operations,
}

impl Hardware {
    /// Creates an instance of the Hardware struct.
    ///
    /// @memory_size: Size of the hardware memory. Max is 65536.
    pub fn new(memory_size: u16) -> Hardware {

        Hardware {
            memory: vec![0; memory_size as usize],
            program_counter: 0,
            stack_pointer: 0,
            registers: [0; 8],
            overflow_flag: false,
            error_flag: false,
            sys_callback: None,
            operations: operations::Operations::new(),
        }
    }

    /// Loads the specified data into memory.
    /// Returns error if data won't fit into memory.
    ///
    /// @data: Data to load.
    /// @start: Memory address to load this memory into.
    pub fn load(&mut self, data: &Vec<u16>, start: u16) -> Result<(), &'static str> {

        // Converting "start" to "usize" for easier usage.
        let start_size: usize = start as usize;

        if start_size + data.len() > self.memory.len() {
            return Err("Out of memory: Data won't fit in memory starting from specified address.");
        }

        // TODO: There should be a faster way.
        for i in 0..data.len() {
            self.memory[start_size + i] = data[i];
        }

        return Ok(());
    }

    /// Executes a clock of CPU.
    /// Returns error only if something really goes wrong
    /// (hardware state is corrupted).
    pub fn clock(&mut self) -> Result<(), String>{

        if self.error_flag {
            return Err(String::from("This hardware is in Error state."));
        }

        // Converting type for easier usage.
        let program_counter = self.program_counter as usize;

        if program_counter >= self.memory.len() {
            return Err(String::from("PC goes beyond the memory!"));
        }

        // Fetching current instruction.
        let instruction = self.memory[program_counter];

        // Executing instruction. Note the "?" (-:
        let executer_function = self.operations.get_function(instruction)?;
        let execute_result = executer_function(self, instruction);

        if execute_result.is_err() {
            // This hardware is no longer in a valid state.
            self.error_flag = true;
            return execute_result;
        }

        // Nothing goes wrong.
        return Ok(());
    }

    /// Increases the memory by the specified additional bytes.
    ///
    /// Returns error if new size would become more than maxed allowed (65536)
    /// Memory won't be touched if error return.
    ///
    /// Returns new size if everything is Ok.
    ///
    /// @additional: Additional bytes to add to the memory size.
    pub fn increase_memory(&mut self, additional: u16) -> Result<u16, &'static str> {

        if additional == 0 {
            return Err("Additional bytes cannot be zero.");
        }

        let current_size = self.memory.len() as u16;
        let new_size = match current_size.checked_add(additional) {
            Some(v) => v,
            None => return Err("New size will become more than 65536 bytes."),
        };

        // For better performance.
        self.memory.reserve(additional as usize);

        // Filling new memory with zeros.
        // TODO: There should be a faster way.
        for _i in current_size..new_size {
            self.memory.push(0u16);
        }

        return Ok(new_size as u16);
    }

    pub fn register_sys_callback(&mut self, callback: Weak<Mutex<SysCallback>>) {
        self.sys_callback = Some(callback);
    }

    pub fn call_syscall(&mut self, cpu_state: &mut CPUState) -> Result<(), &'static str> {

        match self.sys_callback {
            None => return Err("This machine does not support sys calls."),
            Some(ref weak_callback) => {
                // Upgrading Weak to Rc to access its value.
                match weak_callback.upgrade() {
                    // None means this reference is dropped.
                    None => return Err("This machine no longer support sys calls: Callback reference dropped."),
                    // Getting mutable reference and calling the callback.
                    Some(ref mut callback_mutex) => {
                        let mut callback = callback_mutex.lock().
                            expect("Failed to lock the syscall callback. Please report this bug!");
                        callback.syscall(cpu_state);
                    },
                };
            },
        }

        return Ok(());
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::u16;
    use std::rc::Rc;

    /// This method can be used by tests inside other modules to
    /// assert memory of the hardware.
    impl Hardware {
        pub fn compare_memory(&self, expected_memory: &Vec<u16>) {
            assert_eq!(&self.memory, expected_memory);
        }
    }

    #[test]
    fn load() {
        let mut hardware = Hardware::new(12);

        let data = vec![128, 255, 0, 46, 72];

        hardware.load(&data, 7).expect("Failed to load data");

        assert_eq!(hardware.memory[0], 0);
        assert_eq!(hardware.memory[1], 0);
        assert_eq!(hardware.memory[2], 0);
        assert_eq!(hardware.memory[3], 0);
        assert_eq!(hardware.memory[4], 0);
        assert_eq!(hardware.memory[5], 0);
        assert_eq!(hardware.memory[6], 0);
        assert_eq!(hardware.memory[7], 128);
        assert_eq!(hardware.memory[8], 255);
        assert_eq!(hardware.memory[9], 0);
        assert_eq!(hardware.memory[10], 46);
        assert_eq!(hardware.memory[11], 72);

        let data_2 = vec!(72, 0, 0, 1);
        hardware.load(&data_2, 6).expect("Could not load data_2");

        assert_eq!(hardware.memory[0], 0);
        assert_eq!(hardware.memory[1], 0);
        assert_eq!(hardware.memory[2], 0);
        assert_eq!(hardware.memory[3], 0);
        assert_eq!(hardware.memory[4], 0);
        assert_eq!(hardware.memory[5], 0);
        assert_eq!(hardware.memory[6], 72);
        assert_eq!(hardware.memory[7], 0);
        assert_eq!(hardware.memory[8], 0);
        assert_eq!(hardware.memory[9], 1);
        assert_eq!(hardware.memory[10], 46);
        assert_eq!(hardware.memory[11], 72);
    }

    #[test]
    fn load_out_of_memory() {
        let mut hardware = Hardware::new(1024);

        let data = vec![1, 2, 3, 4, 5];
        let load_result = hardware.load(&data, 1022);
        assert_eq!(load_result.is_err(), true);
    }

    #[test]
    fn increase_memory() {
        let mut hardware = Hardware::new(3000);

        assert_eq!(hardware.memory.len(), 3000);

        let new_size = hardware.increase_memory(2500).unwrap();
        assert_eq!(hardware.memory.len(), 5500);
        assert_eq!(new_size, 5500);

        let new_size = hardware.increase_memory(1).unwrap();
        assert_eq!(hardware.memory.len(), 5501);
        assert_eq!(new_size, 5501);

        // Zero error.
        let result = hardware.increase_memory(0);
        assert_eq!(result.is_err(), true);

        // Too big memory error.
        let result = hardware.increase_memory(61000);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn bad_program_counter() {
        let mut hardware = Hardware::new(2000);

        // Equal to size of memory.
        hardware.program_counter = 2000;
        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);

        // Bigger than memory size.
        hardware.program_counter = 2255;
        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_nop() {
        let mut hardware = Hardware::new(3);

        let code = vec![0b0000000000000000u16, 0b0000000000000000u16];
        hardware.load(&code, 0).unwrap();

        assert_eq!(hardware.program_counter, 0);

        hardware.clock().unwrap();

        assert_eq!(hardware.program_counter, 1);
        // Nothing else should be changed.
        assert_eq!(hardware.registers, [0; 8]);


        hardware.clock().unwrap();

        assert_eq!(hardware.program_counter, 2);
        // Nothing else should be changed.
        assert_eq!(hardware.registers, [0; 8]);
    }

    #[test]
    fn instruction_jump() {
        // Testing all four types of addresses.

        let mut hardware = Hardware::new(11);

        let code = vec![0b0000_000001_000010u16, // register 2 is address 3
                        0b0000000000000000u16,
                        0b0000000000000000u16,
                        0b0000_000001_010001u16, // register 1 points to address 7 in memory: 6
                        0b0000000000000000u16,
                        0b0000000000000000u16,
                        0b0000_000001_100011u16, // register 3 (2) + PC (6) = 8 is address
                        0b0000000000000000u16,
                        0b0000_000001_110100u16, // Register 4 (2) + PC (8) = 10 points to address
                        0b0000000000000110u16, // 6
                        0b0000000000000001u16, // 1
                        ];
        hardware.load(&code, 0).unwrap();

        hardware.registers[2] = 3;
        hardware.registers[1] = 9;
        hardware.registers[3] = 2;
        hardware.registers[4] = 2;

        assert_eq!(hardware.program_counter, 0);

        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 3);

        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 6);

        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 8);

        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 1);
    }

    #[test]
    fn instruction_skip_if_zero() {
        let mut hardware = Hardware::new(11);

        let code = vec![0b0000_000010_000000u16, // Register 0
                        0b0000000000000000u16,
                        0b0000_000010_010010u16, // Register 2 -> Memory 10
                        0b0000_000010_110011u16, // Register 3 + PC -> Memory 9
                        0b0000000000000000u16,
                        0b0000_000010_100011u16, // Unsupported address type
                        0b0000000000000000u16,
                        0b0000000000000000u16,
                        0b0000000000000100u16, // Non-zero
                        0b0000000000000000u16, // Zero
                        0b0000000000000100u16, // Non-zero
                        ];
        hardware.load(&code, 0).unwrap();

        hardware.registers[0] = 0;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 2);

        hardware.registers[2] = 10;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 3);

        hardware.registers[3] = 6;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 5);

        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_copy() {
        let mut hardware = Hardware::new(19);

        // -> means "points", => means "copies"
        let code = vec![0b0001_000010_000111u16, // register two => seven
                        0b0001_010011_000110u16, // Register 3 -> memory 9 => register six
                        0b0001_010011_010100u16, // Register 3 -> memory 7 => Register 4 -> memory 12
                        0b0001_110000_000001u16, // Register 0 + PC (3) -> Memory 15 => Register 1
                        0b0001_010101_110110u16, // Register 5 -> Memory 18 => Register 6 + PC (4) -> Memory 17
                        0b0001_100000_000000u16, // Unsupported address type.
                        0b0000_000000_000000u16,
                        // Data
                        1200u16,
                        0u16,
                        2400u16,
                        13u16, // Used as address
                        0u16,
                        1u16,
                        12564u16,
                        0u16,
                        129u16,
                        0u16,
                        8u16,
                        0u16,];
        hardware.load(&code, 0).unwrap();

        // Copying two registers
        hardware.registers[2] = 256;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[7], hardware.registers[2]);
        assert_eq!(hardware.program_counter, 1);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[2], 256);

        hardware.registers[3] = 9;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[6], hardware.memory[9]);
        assert_eq!(hardware.program_counter, 2);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[3], 9);
        assert_eq!(hardware.memory[9], 2400);

        hardware.registers[3] = 7;
        hardware.registers[4] = 12;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[7], hardware.memory[12]);
        assert_eq!(hardware.program_counter, 3);
        // Nothing else should be changed.
        assert_eq!(hardware.memory[7], 1200);
        assert_eq!(hardware.registers[3], 7);
        assert_eq!(hardware.registers[4], 12);

        hardware.registers[0] = 12;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[15], hardware.registers[1]);
        assert_eq!(hardware.program_counter, 4);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[0], 12);
        assert_eq!(hardware.memory[15], 129);

        hardware.registers[5] = 18;
        hardware.registers[6] = 13;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[18], hardware.memory[17]);
        assert_eq!(hardware.program_counter, 5);
        // Nothing else should be changed.
        assert_eq!(hardware.memory[18], 0);
        assert_eq!(hardware.registers[5], 18);
        assert_eq!(hardware.registers[6], 13);

        // Error: Register plus PC is not supported.
        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_add() {
        let mut hardware = Hardware::new(19);

        // -> means "points"
        let code = vec![0b0010_000010_000111u16, // register two + register seven
                        0b0010_010011_000110u16, // Register 3 -> memory 9 + register six
                        0b0010_010011_010100u16, // Register 3 -> memory 7 + Register 4 -> memory 12
                        0b0010_110000_000001u16, // [Register 0 + PC (3)] -> Memory 15 + Register 1
                        0b0010_010101_110110u16, // Register 5 -> Memory 17 + [Register 6 + PC (4)] -> Memory 18
                        0b0010_000100_000100u16, // Register 4 + Register 4
                        0b0010_100000_000000u16, // Unsupported address type.
                        // Data
                        1200u16,
                        0u16,
                        2400u16,
                        13u16, // Used as address
                        0u16,
                        1u16,
                        12564u16,
                        0u16,
                        129u16,
                        0u16,
                        8u16,
                        0u16,];
        hardware.load(&code, 0).unwrap();

        hardware.registers[2] = 256;
        hardware.registers[7] = 100;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[7], 356);
        assert_eq!(hardware.program_counter, 1);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[2], 256);

        hardware.registers[3] = 9;
        hardware.registers[6] = 8000;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[6], 10400);
        assert_eq!(hardware.program_counter, 2);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[3], 9);
        assert_eq!(hardware.memory[9], 2400);

        hardware.registers[3] = 7;
        hardware.registers[4] = 12;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[12], 1201);
        assert_eq!(hardware.program_counter, 3);
        // Nothing else should be changed.
        assert_eq!(hardware.memory[7], 1200);
        assert_eq!(hardware.registers[3], 7);
        assert_eq!(hardware.registers[4], 12);

        hardware.registers[0] = 12;
        hardware.registers[1] = 200;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[1], 329);
        assert_eq!(hardware.program_counter, 4);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[0], 12);
        assert_eq!(hardware.memory[15], 129);

        hardware.registers[5] = 17;
        hardware.registers[6] = 14;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[18], 8);
        assert_eq!(hardware.program_counter, 5);
        // Nothing else should be changed.
        assert_eq!(hardware.memory[17], 8);
        assert_eq!(hardware.registers[5], 17);
        assert_eq!(hardware.registers[6], 14);

        // Saturating add
        hardware.registers[4] = 60000;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[4], u16::MAX);
        assert_eq!(hardware.program_counter, 6);

        // Error: Register plus PC is not supported.
        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_subtract() {
        let mut hardware = Hardware::new(19);

        // -> means "points"
        let code = vec![0b0011_000010_000111u16, // register two - register seven
                        0b0011_010011_000110u16, // Register 3 -> memory 9 - register six
                        0b0011_010011_010100u16, // Register 3 -> memory 7 - Register 4 -> memory 12
                        0b0011_110000_000001u16, // Register 0 + PC (3) -> Memory 15 - Register 1
                        0b0011_010101_110110u16, // Register 5 -> Memory 17 - Register 6 + PC (4) -> Memory 18
                        0b0011_000101_000100u16, // Register 5 -> Memory 8 - Register 4
                        0b0011_100000_000000u16, // Unsupported address type.
                        // Data
                        1200u16,
                        0u16,
                        2400u16,
                        13u16, // Used as address
                        0u16,
                        1u16,
                        12564u16,
                        0u16,
                        129u16,
                        0u16,
                        8u16,
                        0u16,];
        hardware.load(&code, 0).unwrap();

        hardware.registers[2] = 256;
        hardware.registers[7] = 100;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[7], 156);
        assert_eq!(hardware.program_counter, 1);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[2], 256);

        hardware.registers[3] = 9;
        hardware.registers[6] = 1400;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[6], 1000);
        assert_eq!(hardware.program_counter, 2);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[3], 9);
        assert_eq!(hardware.memory[9], 2400);

        hardware.registers[3] = 7;
        hardware.registers[4] = 12;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[12], 1199);
        assert_eq!(hardware.program_counter, 3);
        // Nothing else should be changed.
        assert_eq!(hardware.memory[7], 1200);
        assert_eq!(hardware.registers[3], 7);
        assert_eq!(hardware.registers[4], 12);

        hardware.registers[0] = 12;
        hardware.registers[1] = 29;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[1], 100);
        assert_eq!(hardware.program_counter, 4);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[0], 12);
        assert_eq!(hardware.memory[15], 129);

        hardware.registers[5] = 17;
        hardware.registers[6] = 14;
        hardware.clock().unwrap();
        assert_eq!(hardware.memory[18], 8);
        assert_eq!(hardware.program_counter, 5);
        // Nothing else should be changed.
        assert_eq!(hardware.memory[17], 8);
        assert_eq!(hardware.registers[5], 17);
        assert_eq!(hardware.registers[6], 14);

        hardware.registers[5] = 8;
        hardware.registers[4] = 17;
        hardware.clock().unwrap();
        assert_eq!(hardware.registers[4], 0);
        assert_eq!(hardware.program_counter, 6);
        // Nothing else should be changed.
        assert_eq!(hardware.registers[5], 8);
        assert_eq!(hardware.memory[8], 0);

        // Error: Register plus PC is not supported.
        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_skip_if_equal() {
        let mut hardware = Hardware::new(11);

        let code = vec![0b0100_000000_000001u16, // Register 0 = Register 1
                        0b0000000000000000u16,
                        0b0100_010010_000110u16, // Register 2 -> Memory 8 = Register 6
                        0b0100_110011_010101u16, // Register 3 + PC -> Memory 8 = Register 5 -> Memory 10
                        0b0000000000000000u16,
                        0b0100_000010_100011u16, // Unsupported address type
                        0b0000000000000000u16,
                        0b0000000000000000u16,
                        0b0000000000000100u16, // 4
                        0b0000000000000000u16,
                        0b0000000000000100u16, // 4
        ];
        hardware.load(&code, 0).unwrap();

        hardware.registers[0] = 2000;
        hardware.registers[1] = 2000;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 2);

        hardware.registers[2] = 8;
        hardware.registers[6] = 7;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 3);

        hardware.registers[3] = 5;
        hardware.registers[5] = 10;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 5);

        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_skip_if_greater() {
        let mut hardware = Hardware::new(11);

        let code = vec![0b0101_000000_000001u16, // Register 0 > Register 1
                        0b0000000000000000u16,
                        0b0101_010010_000110u16, // Register 2 -> Memory 8 > Register 6
                        0b0101_110011_010101u16, // Register 3 + PC -> Memory 8 > Register 5 -> Memory 10
                        0b0000000000000000u16,
                        0b0101_000010_100011u16, // Unsupported address type
                        0b0000000000000000u16,
                        0b0000000000000000u16,
                        0b0100000000000101u16, // 16389
                        0b0000000000000000u16,
                        0b0001001001001101u16, // 4685
        ];
        hardware.load(&code, 0).unwrap();

        hardware.registers[0] = 2001;
        hardware.registers[1] = 2000;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 2);

        hardware.registers[2] = 8;
        hardware.registers[6] = 16390;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 3);

        hardware.registers[3] = 5;
        hardware.registers[5] = 10;
        hardware.clock().unwrap();
        assert_eq!(hardware.program_counter, 5);

        let clock_result = hardware.clock();
        assert_eq!(clock_result.is_err(), true);
    }

    #[test]
    fn instruction_set() {
        let mut hardware = Hardware::new(3);

        let code = vec![0b0110_010_001001101u16,
                        0b0110_111_111111111u16,
                        0b0110_010_000000000u16];
        hardware.load(&code, 0).unwrap();


        hardware.registers[4] = 100;
        hardware.registers[3] = 120;
        hardware.registers[1] = 140;
        hardware.registers[0] = 1000;

        hardware.clock().unwrap();
        assert_eq!(hardware.registers[2], 0b0000_000_001001101u16);
        assert_eq!(hardware.program_counter, 1);
        // Nothing else should change.
        assert_eq!(hardware.registers[4], 100);
        assert_eq!(hardware.registers[3], 120);
        assert_eq!(hardware.registers[1], 140);
        assert_eq!(hardware.registers[0], 1000);

        hardware.clock().unwrap();
        assert_eq!(hardware.registers[7], 0b0000_000_111111111u16);
        assert_eq!(hardware.program_counter, 2);
        // Nothing else should change.
        assert_eq!(hardware.registers[2], 0b0000_000_001001101u16);
        assert_eq!(hardware.registers[4], 100);
        assert_eq!(hardware.registers[3], 120);
        assert_eq!(hardware.registers[1], 140);
        assert_eq!(hardware.registers[0], 1000);

        hardware.clock().unwrap();
        assert_eq!(hardware.registers[2], 0u16);
        assert_eq!(hardware.program_counter, 3);
        // Nothing else should change.
        assert_eq!(hardware.registers[7], 0b0000_000_111111111u16);
        assert_eq!(hardware.registers[4], 100);
        assert_eq!(hardware.registers[3], 120);
        assert_eq!(hardware.registers[1], 140);
        assert_eq!(hardware.registers[0], 1000);
    }

    struct MockSyscall {
    }

    impl SysCallback for MockSyscall {
        fn syscall(&mut self, cpu_state: &mut CPUState) {

            if cpu_state.get_register(0) == 1 {
                assert_eq!(cpu_state.get_error_flag(), false);
                cpu_state.set_error_flag(true);
                return;
            }

            assert_eq!(cpu_state.get_error_flag(), false);
            assert_eq!(cpu_state.get_register(0), 17);
            assert_eq!(cpu_state.get_register(1), 128);
            assert_eq!(cpu_state.get_register(7), 5);

            cpu_state.set_register(0, 0);
            cpu_state.set_register(3, 12);
            cpu_state.set_register(7, 2);
        }
    }

    #[test]
    fn instruction_syscall() {
        let mut hardware = Hardware::new(3);

        let syscall_rc = Rc::new(Mutex::new(MockSyscall {}));
        let syscall_weak = Rc::downgrade(&Rc::clone(&syscall_rc));

        hardware.register_sys_callback(syscall_weak);

        let code = vec![0b0000000000_000001u16,
                        0b0000000000_000001u16];
        hardware.load(&code, 0).unwrap();

        hardware.registers[0] = 17;
        hardware.registers[1] = 128;
        hardware.registers[7] = 5;

        hardware.clock().unwrap();

        assert_eq!(hardware.registers[0], 0);
        assert_eq!(hardware.registers[3], 12);
        assert_eq!(hardware.registers[7], 2);

        hardware.registers[0] = 1;
        let clock_result = hardware.clock();

        assert_eq!(clock_result.is_err(), true);
        assert_eq!(hardware.error_flag, true);
    }

}
