/// This module implements the hardware of the emulator.
/// It emulates CPU and memory of a single machine.

struct Hardware {
    memory: Vec<u16>,

    program_counter: u16,
    stack_pointer: u8,

    // There are 8 registers.
    registers: [u16; 8],

    overflow_flag: bool,
    error_flag: bool,

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

        let instruction = self.memory[program_counter];

        // Finding type of instruction.

        if instruction & 0b1111111111000000u16 == 0b0000000000000000u16 {
            self.execute_no_operand_instruct(instruction);
        }
        else if instruction & 0b1111000000000000u16 == 0b0000000000000000u16 {
            self.execute_single_operand_instruct(instruction);
        }
        else {
            self.execute_double_operand_instruct(instruction);
        }

        return Ok(());
    }

    fn execute_no_operand_instruct(&mut self, instruction: u16) {

        // NOP
        if instruction == 0b0000000000000000u16 {
            self.program_counter += 1;
        }
    }

    fn execute_single_operand_instruct(&mut self, instruction: u16) {

    }

    fn execute_double_operand_instruct(&mut self, instruction: u16) {

    }

    /// Gets operand, from the specified address.
    fn get_operand(&self, address: u8) -> Result<u16, String> {

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
            return Ok(self.registers[address_value]);
        }
        else if address_type == 0b00_01_0000u8 {
            // Register is address of operand.
            let operand_address = self.registers[address_value] as usize;

            if operand_address >= self.memory.len() {
                return Err(format!(
                    "Address is out of memory. Address was [{}] stored in register [{}].",
                    operand_address, address_value));
            }

            return Ok(self.memory[operand_address]);
        }
        else if address_type == 0b00_10_0000u8 {
            // Register + Program Counter is the operand.
            // We used "saturating_add" so result will be max of u16 if value becomes
            // too large.
            return Ok(self.registers[address_value].saturating_add(self.program_counter));
        }
        else if address_type == 0b00_11_0000u8 {
            // Register + Program Counter is the address of operand.
            let (operand_address, is_overflowed) =
                self.registers[address_value].overflowing_add(self.program_counter);
            let operand_address = operand_address as usize;

            if is_overflowed {
                return Err(format!(
                    "Memory address overflow. PC ({}) + Register{} ({})",
                    self.program_counter, address_value, self.registers[address_value]))
            }

            if operand_address >= self.memory.len() {
                return Err(format!(
                    "Address is out of memory. Address was [{}] stored in register [{}].",
                    operand_address, address_value));
            }

            return Ok(self.memory[operand_address]);
        }

        // We checked all of possibilities. We should never reach here.
        return Err(format!(
            "Unknown memory address type. Address was: [{:b}] This is a bug! Please report it.",
            address));
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
//        assert_eq!(hardware.register_0, 0);
//        assert_eq!(hardware.register_1, 0);
//        assert_eq!(hardware.register_2, 0);
//        assert_eq!(hardware.register_3, 0);
//        assert_eq!(hardware.register_4, 0);
//        assert_eq!(hardware.register_5, 0);
//        assert_eq!(hardware.register_6, 0);
//        assert_eq!(hardware.register_7, 0);

        hardware.clock().unwrap();

        assert_eq!(hardware.program_counter, 2);
        // Nothing else should be changed.
        assert_eq!(hardware.registers, [0; 8]);
    }

    /// Tests address type one.
    #[test]
    fn get_operand_type_one() {

        let mut hardware = Hardware::new(1);

        hardware.registers[0] = 128;
        hardware.registers[5] = 254;
        hardware.registers[7] = 1;

        let operand = hardware.get_operand(0b0000_0000u8).unwrap();
        assert_eq!(operand, 128);

        let operand = hardware.get_operand(0b0000_0101u8).unwrap();
        assert_eq!(operand, 254);

        let operand = hardware.get_operand(0b0000_0111u8).unwrap();
        assert_eq!(operand, 1);

        // Bad register address.
        let operand = hardware.get_operand(0b0000_1000u8);
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

        let operand = hardware.get_operand(0b0001_0000u8).unwrap();
        assert_eq!(operand, 128);
        let operand = hardware.get_operand(0b0001_0101u8).unwrap();
        assert_eq!(operand, 64);
        let operand = hardware.get_operand(0b0001_0111u8).unwrap();
        assert_eq!(operand, 1);

        // Bad address.
        hardware.registers[1] = 14;
        let operand = hardware.get_operand(0b0001_0001u8);
        assert_eq!(operand.is_err(), true);
    }

    /// Tests address type one.
    #[test]
    fn get_operand_type_three() {

        let mut hardware = Hardware::new(1);

        hardware.program_counter = 9;
        hardware.registers[2] = 7;
        let operand = hardware.get_operand(0b0010_0010u8).unwrap();
        assert_eq!(operand, 16);

        hardware.program_counter = 0;
        hardware.registers[1] = 18;
        let operand = hardware.get_operand(0b0010_0001u8).unwrap();
        assert_eq!(operand, 18);

        hardware.program_counter = 12;
        hardware.registers[3] = 0;
        let operand = hardware.get_operand(0b0010_0011u8).unwrap();
        assert_eq!(operand, 12);

        // Saturating add
        hardware.program_counter = 65000;
        hardware.registers[6] = 17000;
        let operand = hardware.get_operand(0b0010_0110u8).unwrap();
        assert_eq!(operand, 65535u16);
    }

    /// Tests address type one.
    #[test]
    fn get_operand_type_four() {

        let mut hardware = Hardware::new(20);

        hardware.program_counter = 9;
        hardware.registers[2] = 7;
        hardware.memory[16] = 120;
        let operand = hardware.get_operand(0b0011_0010u8).unwrap();
        assert_eq!(operand, 120);

        hardware.program_counter = 0;
        hardware.registers[1] = 18;
        hardware.memory[18] = 254;
        let operand = hardware.get_operand(0b0011_0001u8).unwrap();
        assert_eq!(operand, 254);

        hardware.program_counter = 12;
        hardware.registers[3] = 0;
        hardware.memory[12] = 107;
        let operand = hardware.get_operand(0b0011_0011u8).unwrap();
        assert_eq!(operand, 107);

        // Overflow
        hardware.program_counter = 65000;
        hardware.registers[6] = 17000;
        let operand = hardware.get_operand(0b0011_0110u8);
        assert_eq!(operand.is_err(), true);

        // Out of memory.
        hardware.program_counter = 11;
        hardware.registers[0] = 10;
        let operand = hardware.get_operand(0b0011_0000u8);
        assert_eq!(operand.is_err(), true);
    }
}
