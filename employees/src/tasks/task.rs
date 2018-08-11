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

use rand::prelude::thread_rng;
use rand::Rng;

pub struct Task {
    id: u16,
    code: Vec<u16>,
    expected_result: u16,
}

/// Creates and returns a random small task (13 instructions).
pub fn make_small_task() -> Task {

    // Three random numbers in DATA.
    // add first one to the second one, three times.
    // subtract third one from the above result, two times.
    // Result is in register two.
    // Return.

    let mut code: Vec<u16> = Vec::with_capacity(13);
    let mut rng = thread_rng();

    code.push(0b0110_100_000001001u16); // Set R4 to 9
    code.push(0b0001_11_0100_00_0001u16); // Copy memory in 9+PC to R1 (9 is the value of R4)
    code.push(0b0001_11_0100_00_0010u16); // Copy memory in 9+PC to R2
    code.push(0b0001_11_0100_00_0011u16); // Copy memory in 9+PC to R3
    code.push(0b0010_00_0001_00_0010u16); // Add R1 to R2
    code.push(0b0010_00_0001_00_0010u16); // Add R1 to R2
    code.push(0b0010_00_0001_00_0010u16); // Add R1 to R2
    code.push(0b0011_00_0010_00_0011u16); // Subtract R2 from R3
    code.push(0b0011_00_0010_00_0011u16); // Subtract R2 from R3
    code.push(0b0000000000_000010u16); // Return

    // Three random numbers as data.
    let d1: u16 = rng.gen_range(1, 1000);
    let d2: u16 = rng.gen_range(1000, 2000); // It's at least 1000 to prevent underflow of subtraction.
    let d3: u16 = rng.gen_range(1, 500);

    code.push(d1);
    code.push(d2);
    code.push(d3);

    let expected: u16 = (d1 + d1 + d1 + d2) - (d3 + d3);

    Task {
        id: rng.gen_range(1, 65534),
        code: code,
        expected_result: expected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emulator::Emulator;

    /// Runs five random small tasks
    #[test]
    fn five_random_small_tasks() {
        for i in 0..5 {
            let task: Task = make_small_task();

            let mut emu = Emulator::new(15);
            let mut code = task.code.clone();
            // Adding a `subroutine' to as the first instruction, because the last one is `return'.
            code.insert(0, 0b0110_000_000000011u16); // Set R0 to 3
            code.insert(1, 0b0000_000011_000000u16); // Subroutine to R0 (3)

            emu.load(&code, 0);

            for i in 0..15 {
                emu.clock().unwrap()
            }

            // TODO: Find a way to validate the result (compare R2 with task.expected_result)
        }
    }
}
