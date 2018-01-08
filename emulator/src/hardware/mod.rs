/// This module implements the hardware of the emulator.
/// It emulates CPU and memory of a single machine.

mod operations;
mod operation_code;


pub struct Hardware {
    memory: Vec<u16>,

    program_counter: u16,
    stack_pointer: u8,

    // There are 8 registers.
    registers: [u16; 8],

    overflow_flag: bool,
    error_flag: bool,

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

        // Converting type for easier usage.
        let program_counter = self.program_counter as usize;

        if program_counter >= self.memory.len() {
            return Err(String::from("PC goes beyond the memory!"));
        }

        // Fetching current instruction.
        let instruction = self.memory[program_counter];

        // Executing instruction. Note the "?" (-:
        let executer_function = self.operations.get_function(instruction)?;
        executer_function(self, instruction)?;

        // Nothing goes wrong.
        return Ok(());
    }

}


#[cfg(test)]
mod tests {

    use super::*;
    use std::u16;

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

        let code = vec![0b0000_000010_000001u16, // Register 0
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
}
