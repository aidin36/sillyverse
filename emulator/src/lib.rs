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

/// This module provides an interface to the library.

mod hardware;
mod cpu_state;
mod sys_callback;

use std::rc::Weak;
use std::sync::Mutex;

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

    /// Executes a clock of CPU.
    /// Returns error only if something really goes wrong
    /// (hardware state is corrupted).
    pub fn clock(&mut self) -> Result<(), String> {
        return self.hardware.clock();
    }

    /// Registers a callback function that is responsible for handling sys calls.
    pub fn register_sys_callback(&mut self, callback: Weak<Mutex<SysCallback>>) {
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