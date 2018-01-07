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
}
