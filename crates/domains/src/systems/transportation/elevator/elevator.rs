/// Direction an elevator is traveling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Idle,
}

/// Door state — must be closed before moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoorState {
    Open,
    Closed,
}

/// A single elevator car with its current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elevator {
    pub id: usize,
    pub floor: usize,
    pub num_floors: usize,
    pub direction: Direction,
    pub door: DoorState,
    pub load: u32,     // current weight in kg
    pub capacity: u32, // max weight in kg
    /// Floors this elevator has been requested to stop at.
    pub stops: Vec<usize>,
}

impl Elevator {
    pub fn new(id: usize, capacity: u32, num_floors: usize) -> Self {
        Self {
            id,
            floor: 0,
            num_floors,
            direction: Direction::Idle,
            door: DoorState::Closed,
            load: 0,
            capacity,
            stops: Vec::new(),
        }
    }

    /// Can this elevator accept more load?
    pub fn can_accept(&self, weight: u32) -> bool {
        self.load + weight <= self.capacity
    }

    /// Is the elevator currently idle (no stops, not moving)?
    pub fn is_idle(&self) -> bool {
        self.direction == Direction::Idle && self.stops.is_empty()
    }

    /// Distance to a given floor.
    pub fn distance_to(&self, floor: usize) -> usize {
        (self.floor as isize - floor as isize).unsigned_abs()
    }

    /// Does this elevator need to stop at its current floor?
    pub fn should_stop_here(&self) -> bool {
        self.stops.contains(&self.floor)
    }

    /// Add a stop. Returns false if the stop violates direction commitment or is out of range.
    pub fn add_stop(&mut self, floor: usize) -> bool {
        if floor >= self.num_floors {
            return false;
        }
        if self.stops.contains(&floor) {
            return true; // already scheduled
        }
        // Direction commitment: if moving up, only accept floors above (or equal)
        // If moving down, only accept floors below (or equal)
        // If idle, accept anything
        match self.direction {
            Direction::Up if floor < self.floor => return false,
            Direction::Down if floor > self.floor => return false,
            _ => {}
        }
        self.stops.push(floor);
        self.stops.sort();
        true
    }

    /// Open doors. Only valid when stopped (not between floors).
    pub fn open_doors(&mut self) -> Result<(), &'static str> {
        if self.door == DoorState::Open {
            return Err("doors already open");
        }
        self.door = DoorState::Open;
        // Remove this floor from stops
        self.stops.retain(|&f| f != self.floor);
        Ok(())
    }

    /// Close doors.
    pub fn close_doors(&mut self) -> Result<(), &'static str> {
        if self.door == DoorState::Closed {
            return Err("doors already closed");
        }
        self.door = DoorState::Closed;
        Ok(())
    }

    /// Move one floor in the current direction.
    /// Enforces: doors must be closed, must have a direction.
    pub fn move_one(&mut self) -> Result<(), &'static str> {
        if self.door == DoorState::Open {
            return Err("cannot move with doors open");
        }
        match self.direction {
            Direction::Up => {
                if self.floor + 1 >= self.num_floors {
                    return Err("already at top floor");
                }
                self.floor += 1;
            }
            Direction::Down => {
                if self.floor == 0 {
                    return Err("already at ground floor");
                }
                self.floor -= 1;
            }
            Direction::Idle => return Err("no direction set"),
        }

        // Auto-update direction when all stops in current direction are served
        self.update_direction();
        Ok(())
    }

    /// Board passengers (add load).
    pub fn board(&mut self, weight: u32) -> Result<(), &'static str> {
        if self.door != DoorState::Open {
            return Err("doors must be open to board");
        }
        if self.load + weight > self.capacity {
            return Err("exceeds capacity");
        }
        self.load += weight;
        Ok(())
    }

    /// Unload passengers (remove load).
    pub fn unload(&mut self, weight: u32) -> Result<(), &'static str> {
        if self.door != DoorState::Open {
            return Err("doors must be open to unload");
        }
        if weight > self.load {
            return Err("cannot unload more than current load");
        }
        self.load -= weight;
        Ok(())
    }

    /// Update direction based on remaining stops.
    fn update_direction(&mut self) {
        if self.stops.is_empty() {
            self.direction = Direction::Idle;
            return;
        }
        let has_above = self.stops.iter().any(|&f| f > self.floor);
        let has_below = self.stops.iter().any(|&f| f < self.floor);

        self.direction = match self.direction {
            Direction::Up if has_above => Direction::Up,
            Direction::Up if has_below => Direction::Down,
            Direction::Down if has_below => Direction::Down,
            Direction::Down if has_above => Direction::Up,
            Direction::Idle if has_above => Direction::Up,
            Direction::Idle if has_below => Direction::Down,
            _ => Direction::Idle,
        };
    }
}
