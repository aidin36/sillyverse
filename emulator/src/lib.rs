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

/// This module provides an interface to the library.

mod hardware;
mod cpu_state;
mod sys_callback;

use std::rc::Weak;
use std::sync::Mutex;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;


// Importing public API types.
pub use cpu_state::CPUState;
pub use sys_callback::SysCallback;


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

    pub fn load_from_file(&mut self, file_path: &String, start: u16) -> Result<(), &'static str> {
        let file = match File::open(&file_path) {
            Ok(f) => f,
            Err(ioerror) => {
                eprintln!("Error opening file [{}]: {}", file_path, ioerror);
                return Err("Could not open specified file. See stderr for the error.");
            },
        };

        let mut reader = BufReader::new(&file);
        let mut instruction: [u8; 2] = [0; 2];
        let mut data: Vec<u16> = Vec::new();

        loop {
            let read_size = match reader.read(&mut instruction) {
                Ok(size) => size,
                Err(ioerror) => {
                    eprintln!("Error reading file [{}]: {}", file_path, ioerror);
                    return Err("Could not read from file. See stderr for the error.");
                },
            };

            if read_size == 0 {
                // End of file.
                break;
            }

            if read_size == 1 {
                return Err("File should be multiply of two-bytes.");
            }

            data.push(((instruction[0] as u16) << 8) | (instruction[1] as u16));
        }

        return self.load(&data, start);
    }

    /// Executes a clock of CPU.
    /// Returns error only if something really goes wrong
    /// (hardware state is corrupted).
    pub fn clock(&mut self) -> Result<(), String> {
        return self.hardware.clock();
    }

    /// Registers a callback function that is responsible for handling sys calls.
    pub fn register_sys_callback(&mut self, callback: Weak<Mutex<dyn SysCallback>>) {
        self.hardware.register_sys_callback(callback);
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

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Write;
    use std::env::temp_dir;

    #[test]
    fn load_from_file() {
        let mut code_file = temp_dir();
        code_file.push("test_binary_code_1_i73yvfc");

        let mut f = File::create(&code_file).unwrap();

        let code = vec![0b00000000u8, 0b00000000u8, 0b00010000u8, 0b01000010u8,
                        0b00100000u8, 0b10110011u8, 0b01101001u8, 0b11111111u8,
                        0b00000000u8, 0b00000000u8, 0b00001011u8, 0b10111000u8];
        f.write(&code).unwrap();
        f.flush().unwrap();

        let mut emulator = Emulator::new(8);
        emulator.load_from_file(&String::from(code_file.to_str().unwrap()), 2).unwrap();

        // Load starts from 2 index, so first two words are zero too.
        let expected_memory = vec![0b0000000000000000u16, 0b0000000000000000u16,
                                   0b0000000000000000u16, 0b0001000001000010u16,
                                   0b0010000010110011u16, 0b0110100111111111u16,
                                   0b0000000000000000u16, 0b0000101110111000u16];
        emulator.hardware.compare_memory(&expected_memory);
    }
}
