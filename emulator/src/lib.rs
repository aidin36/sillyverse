/// This module provides an interface to the library.

mod hardware;

pub struct Emulator {
    hardware: hardware::Hardware,
}

impl Emulator {

    /// Creates an instance of the Hardware struct.
    ///
    /// @memory_size: Size of the hardware memory. Max is 65536.
    pub fn new(memory_size: u16) -> Emulator {
        Emulator {
            hardware: hardware::Hardware::new(memory_size),
        }
    }

    /// Loads the specified data into memory.
    /// Returns error if data won't fit into memory.
    ///
    /// @data: Data to load.
    /// @start: Memory address to load this memory into.
    pub fn load(&mut self, data: &Vec<u16>, start: u16) -> Result<(), &'static str> {
        return self.hardware.load(data, start);
    }

    /// Executes a clock of CPU.
    /// Returns error only if something really goes wrong
    /// (hardware state is corrupted).
    pub fn clock(&mut self) -> Result<(), String> {
        return self.hardware.clock();
    }

    pub fn register_sys_callback(&mut self) -> Result<(), String> {
        return Err(String::from("Not implemented."));
    }

    /// Increases the memory by the specified additional bytes.
    ///
    /// Returns error if new size would become more than maxed allowed (65536)
    /// Memory won't be touched if error return.
    ///
    /// @additional: Additional bytes to add to the memory size.
    pub fn increase_memory(&mut self, additional: u16) -> Result<u16, &'static str> {
        return self.hardware.increase_memory(additional);
    }

}