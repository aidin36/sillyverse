/// This module implements the hardware of the emulator.
/// It emulates CPU and memory of a single machine.

struct Hardware {
    memory: Vec<u16>,

    pc_register: u16,
    stack_pointer: u8,

    register_0: u16,
    register_1: u16,
    register_2: u16,
    register_3: u16,
    register_4: u16,
    register_5: u16,
    register_6: u16,
    register_7: u16,

    overflow_flag: bool,
    error_flag: bool,

}

impl Hardware {
    /// Creates an instance of the Hardware struct.
    ///
    /// @memory_size: Size of the hardware memory. Max is 65536.
    pub fn new(memory_size: u16) -> Result<Hardware, &'static str> {

        let instance = Hardware {
            memory: vec![0; memory_size as usize],
            pc_register: 0,
            stack_pointer: 0,
            register_0: 0,
            register_1: 0,
            register_2: 0,
            register_3: 0,
            register_4: 0,
            register_5: 0,
            register_6: 0,
            register_7: 0,
            overflow_flag: false,
            error_flag: false,
        };

        return Ok(instance);
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
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load() {
        let mut hardware = Hardware::new(12).unwrap();

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
        let mut hardware = Hardware::new(1024).unwrap();

        let data = vec![1, 2, 3, 4, 5];
        let load_result = hardware.load(&data, 1022);
        assert_eq!(load_result.is_err(), true);
    }
}