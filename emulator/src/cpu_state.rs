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


/// Holds current state of the CPU.
/// This struct is used in the public API of the library.
pub struct CPUState {
    registers: [u16; 8],
    error_flag: bool,
}

impl CPUState {

    // Creates a new instance of CPUState. It clones the registers.
    pub fn new(registers: &[u16; 8]) -> CPUState {

        CPUState {
            registers: registers.clone(),
            error_flag: false,
        }
    }

    pub fn get_error_flag(&self) -> bool {
        return self.error_flag;
    }

    pub fn set_error_flag(&mut self, value: bool) {
        self.error_flag = value;
    }

    pub fn get_register(&self, index: usize) -> u16 {
        return self.registers[index];
    }

    pub fn set_register(&mut self, index: usize, value: u16) {
        self.registers[index] = value;
    }
}