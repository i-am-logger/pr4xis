#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::op::{BinaryOp, UnaryOp};
use super::value::{AngleMode, CalcError, Value};

/// Memory operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOp {
    Store,  // M = current
    Recall, // current = M
    Add,    // M += current
    Clear,  // M = 0
}

/// A stateful scientific calculator with full enforcement.
#[derive(Debug, Clone, PartialEq)]
pub struct Calculator {
    pub display: Value,
    pub memory: Value,
    pub angle_mode: AngleMode,
    pub history: Vec<(String, Value)>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            display: Value::int(0),
            memory: Value::int(0),
            angle_mode: AngleMode::Radians,
            history: Vec::new(),
        }
    }

    /// Set a value on the display.
    pub fn enter(&mut self, value: Value) {
        self.display = value;
    }

    /// Apply a unary operation to the display value.
    pub fn unary(&mut self, op: UnaryOp) -> Result<(), CalcError> {
        let result = op.apply(&self.display, self.angle_mode)?;
        let desc = format!("{:?}({})", op, self.display);
        self.history.push((desc, result.clone()));
        self.display = result;
        Ok(())
    }

    /// Apply a binary operation: display = display op value.
    pub fn binary(&mut self, op: BinaryOp, rhs: Value) -> Result<(), CalcError> {
        let result = op.apply(&self.display, &rhs)?;
        let desc = format!("{} {:?} {}", self.display, op, rhs);
        self.history.push((desc, result.clone()));
        self.display = result;
        Ok(())
    }

    /// Memory operation.
    pub fn memory_op(&mut self, op: MemoryOp) {
        match op {
            MemoryOp::Store => self.memory = self.display.clone(),
            MemoryOp::Recall => self.display = self.memory.clone(),
            MemoryOp::Add => {
                if let Ok(sum) = BinaryOp::Add.apply(&self.memory, &self.display) {
                    self.memory = sum;
                }
            }
            MemoryOp::Clear => self.memory = Value::int(0),
        }
    }

    /// Switch angle mode.
    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.angle_mode = mode;
    }

    /// Clear display to 0.
    pub fn clear(&mut self) {
        self.display = Value::int(0);
    }

    /// Clear everything (display, memory, history).
    pub fn all_clear(&mut self) {
        self.display = Value::int(0);
        self.memory = Value::int(0);
        self.history.clear();
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
