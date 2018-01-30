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

#[macro_use]
extern crate log;
extern crate simplelog;

extern crate emulator;

mod machine;
mod syscalls;

use std::env;
use std::sync::Mutex;
use std::rc::Rc;
use std::process;


/// Starts the game.
///
/// @bots: List of paths to binary files of bots.
/// @initial_memory: Initial memory for each bot's machine.
/// @initial_credit: Initial credit for each bot.
fn start(bots: &Vec<String>, initial_memory: u16, initial_credit: u16) {

    let mut machines: Vec<Rc<Mutex<machine::Machine>>> = Vec::with_capacity(bots.len());

    // Creating a machine for each bot.
    for bot in bots.iter() {
        let bot_machine =
            machine::Machine::new(bot, initial_memory, initial_credit);
        bot_machine.lock().unwrap().load_bot(bot).
            expect("Could not load bot.");

        machines.push(bot_machine);
    }

    // Main loop
    loop {
        // Keeps index of bots that should be removed from the list (dead bots).
        let mut bots_to_remove: Vec<usize> = Vec::new();

        for (index, bot_machine_mutex) in machines.iter().enumerate() {
            let mut bot_machine = bot_machine_mutex.lock().unwrap();
            let result = bot_machine.clock();
            if result.is_err() {
                error!("{}", result.unwrap_err());
                error!("let it die.");
                bots_to_remove.push(index);
            }
        }

        if !bots_to_remove.is_empty() {
            // Removing dead bots.
            // We iterates in reverse order, because "remove" will
            // change indexes.
            for index in bots_to_remove.iter().rev() {
                machines.remove(*index);
            }
        }

        if machines.is_empty() {
            info!("No bot remained alive!");
            break;
        }

        if machines.len() == 1 {
            info!("Only one bot remained alive! Our lucky winner: [{}]",
                     machines.get(0).unwrap().lock().unwrap().get_name());
            break;
        }
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    // First arg is the binary itself.
    args.remove(0);

    if args.is_empty() {
        println!("No bot specified!");
        println!("Usage: employees path_to_bot_file_1 path_to_bot_file_2 ...");
        process::exit(1);
    }

    // Configuring logger.
    // TODO: Read logging configs from a file.
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info, simplelog::Config::default())
        .unwrap();

    //TODO: Read initial values from config file.
    start(&args, 128, 80);

    info!("The game finished.");
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::io;
    use std::env::temp_dir;
    use std::sync::{Mutex, Arc};
    use std::ops::Deref;

    struct MockLogger {
        logs_buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for MockLogger {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.logs_buffer.lock().unwrap().extend_from_slice(buf);
            return Ok(buf.len());
        }

        fn flush(&mut self) -> io::Result<()> {
            return Ok(());
        }
    }

    #[test]
    fn two_bots() {
        let mut first_bot_file_path = temp_dir();
        first_bot_file_path.push("test_first_bot_kjf8743");
        let mut first_bot_file = File::create(&first_bot_file_path).unwrap();
        first_bot_file.write(&[0b01100010u8, 0b01111000u8, // SET R1 120
                               0b01100100u8, 0b10001100u8, // SET R2 140
                               0b00100000u8, 0b01000010u8, // ADD R1 R2
                              ]);
        first_bot_file.flush();

        let mut second_bot_file_path = temp_dir();
        second_bot_file_path.push("test_second_bot_fju8734");
        let mut second_bot_file = File::create(&second_bot_file_path).unwrap();
        second_bot_file.write(&[0b00000000u8, 0b00000000u8, // NOP
                                     0b11110011u8, 0b11111111u8, // Bad instruction
                                     ]);
        second_bot_file.flush();

        let first_bot_file_path = String::from(first_bot_file_path.to_str().unwrap());
        let second_bot_file_path = String::from(second_bot_file_path.to_str().unwrap());

        // Initializing the logger in a way we can check the output later.
        let logs_buffer = Arc::new(Mutex::new(Vec::new()));
        let mock_logger = MockLogger {
            logs_buffer: Arc::clone(&logs_buffer),
        };

        simplelog::WriteLogger::init(simplelog::LogLevelFilter::Info,
                                     simplelog::Config::default(),
                                     mock_logger);

        start(&vec![first_bot_file_path.clone(), second_bot_file_path.clone()],
              20, 3);

        let expected_log_1 = format!("Error in machine [{}]: Unknown instruction: [1111001111111111]", second_bot_file_path);
        let expected_log_2 = format!("Only one bot remained alive! Our lucky winner: [{}]", first_bot_file_path);

        let logs = String::from_utf8(logs_buffer.lock().unwrap().clone()).unwrap();
        assert!(logs.contains(expected_log_1.as_str()));
        assert!(logs.contains(expected_log_2.as_str()))
    }
}