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

use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq)]
enum OperationType {
    NoOperand,
    SingleOperand,
    DoubleOperand,
}

/// This structures holds an entry of the memory that represents
/// an operation. It is implemented Eq and Hash to work correctly
/// in a HashMap.
pub struct OperationCode {
    value: u16,
}

impl OperationCode {
    pub fn new(value: u16) -> OperationCode {
        OperationCode {
            value: value,
        }
    }

    /// Returns operation type of the specified value.
    fn get_operation_type(&self, instruction: u16) -> OperationType {

        if instruction & 0b1111111111000000u16 == 0b0000000000000000u16 {
            return OperationType::NoOperand;
        } else if instruction & 0b1111000000000000u16 == 0b0000000000000000u16 {
            return OperationType::SingleOperand;
        } else {
            return OperationType::DoubleOperand;
        }
    }

    /// Gets a mask that extracts operation code from the specified value.
    fn get_operation_mask(&self, operation_type: OperationType) -> u16 {

        match operation_type {
            OperationType::NoOperand => return 0b0000000000111111u16,
            OperationType::SingleOperand => return 0b0000111111000000u16,
            OperationType::DoubleOperand => return 0b1111000000000000u16,
        }
    }
}

impl PartialEq for OperationCode {
    fn eq(&self, other: &OperationCode) -> bool {

        // Comparing types, then comparing operation type without its operands.
        let my_type = self.get_operation_type(self.value);
        let other_type = self.get_operation_type(other.value);

        if my_type != other_type {
            return false;
        }

        let mask = self.get_operation_mask(my_type);
        if self.value & mask == other.value & mask {
            return true;
        }

        return false;
    }
}

impl Eq for OperationCode {}

impl Hash for OperationCode {
    fn hash<H: Hasher>(&self, state: &mut H) {

        // Extracting operation code without its operands, and hash that.
        let mask = self.get_operation_mask(self.get_operation_type(self.value));

        (self.value & mask).hash(state);
    }
}
