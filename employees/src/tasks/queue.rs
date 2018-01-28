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

use std::collections::HashMap;
use tasks::task::Task;

/// Defined queue of tasks.
pub struct TasksQueue {
    issued_tasks: HashMap<u16, Task>
}

impl TasksQueue {

    pub fn new() -> TasksQueue {
        TasksQueue {
            issued_tasks: HashMap::new(),
        }
    }

    pub fn get_task() {

    }
}