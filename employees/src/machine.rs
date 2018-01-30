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

use std::rc::Rc;
use std::sync::Mutex;
use emulator::Emulator;
use emulator::CPUState;
use emulator::SysCallback;
use syscalls;


pub struct Machine {
    name: String,
    emulator: Emulator,
    credit: u16,
}

impl Machine {

    pub fn new(name: &String, memory_size: u16, initial_credit: u16) -> Rc<Mutex<Machine>> {
        let instance = Machine {
            name: name.clone(),
            emulator: Emulator::new(memory_size),
            credit: initial_credit,
        };

        let rc_instance = Rc::new(Mutex::new(instance));

        // Passing a weak reference of instance to the emulator.
        let weak_instance = Rc::downgrade(&Rc::clone(&rc_instance));
        rc_instance.lock().unwrap().emulator.register_sys_callback(weak_instance);

        return rc_instance;
    }

    /// Gets name of the machine.
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    /// Loads a bot into the machine.
    /// It loads the bot into the zero index of the memory.
    ///
    /// @file_path: Path to the file that contains bot's binary code.
    pub fn load_bot(&mut self, file_path: &String) -> Result<(), &'static str> {
        return self.emulator.load_from_file(file_path, 0);
    }

    /// Clocks the machine CPU.
    /// If any error returns, it means something went really wrong and
    /// this machine is no longer in a valid state.
    pub fn clock(&mut self) -> Result<(), String> {
        let result = self.emulator.clock();

        if result.is_err() {
            return Err(format!("Error in machine [{}]: {}", self.name, result.unwrap_err()));
        }

        self.credit -= 1;

        if self.credit == 0 {
            return Err(format!("This machine has no more credit: [{}]", self.name));
        }

        return Ok(());
    }

}

impl SysCallback for Machine {

    fn syscall(&mut self, cpu_state: &mut CPUState) {
        syscalls::handle_syscall(self, cpu_state);
    }
}
