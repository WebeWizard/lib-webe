#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use webe_id::time::Clock;

#[derive(Clone, Debug)]
pub struct ManualClock {
    current: Arc<Mutex<SystemTime>>,
}

impl ManualClock {
    pub fn new(current: SystemTime) -> Self {
        Self {
            current: Arc::new(Mutex::new(current)),
        }
    }

    pub fn set(&self, current: SystemTime) {
        *self.current.lock().unwrap() = current;
    }

    pub fn advance(&self, duration: Duration) {
        let mut current = self.current.lock().unwrap();
        *current += duration;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> SystemTime {
        *self.current.lock().unwrap()
    }
}

#[derive(Debug)]
pub struct StepClock {
    current: Mutex<SystemTime>,
    step: Duration,
}

impl StepClock {
    pub fn new(current: SystemTime, step: Duration) -> Self {
        Self {
            current: Mutex::new(current),
            step,
        }
    }
}

impl Clock for StepClock {
    fn now(&self) -> SystemTime {
        let mut current = self.current.lock().unwrap();
        let observed = *current;
        *current += self.step;
        observed
    }
}

pub fn epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH
}

pub fn at_ms(milliseconds: u64) -> SystemTime {
    epoch() + Duration::from_millis(milliseconds)
}
