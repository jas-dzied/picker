use std::{
    fmt,
    io::{self, Write},
    time::{Duration, Instant},
};

use crate::Solution;
use anyhow::Result;
use colored::Colorize;

pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
}

impl TimeUnit {
    pub fn next(&self) -> Self {
        match self {
            Self::Nanoseconds => Self::Microseconds,
            Self::Microseconds => Self::Milliseconds,
            Self::Milliseconds => Self::Seconds,
            Self::Seconds => unreachable!(),
        }
    }
    pub const fn repr(&self) -> &str {
        match self {
            Self::Nanoseconds => "ns",
            Self::Microseconds => "μs",
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
        }
    }
}

fn display_duration(duration: Duration) -> (u128, TimeUnit) {
    let mut unit = TimeUnit::Nanoseconds;
    let mut time_since_start = duration.as_nanos();
    if time_since_start > 5000 {
        time_since_start /= 1000;
        unit = unit.next();
    }
    if time_since_start > 5000 {
        time_since_start /= 1000;
        unit = unit.next();
    }
    if time_since_start > 5000 {
        time_since_start /= 1000;
        unit = unit.next();
    }
    (time_since_start, unit)
}

pub struct Logger {
    start: Instant,
}

pub fn display_result(solution: &Solution) {
    println!("{}", "RESULT".green().bold());
    println!(
        " preferred matchups:    {}",
        solution.preferred.to_string().blue()
    );
    println!(
        " accepted matchups:     {}",
        solution.accepted.to_string().blue()
    );
    println!(
        " unpreferred matchups:  {}",
        solution.unpreferred.to_string().blue()
    );
    for room in &solution.rooms {
        println!(" {} & {}", room.0.blue(), room.1.blue());
    }
}

pub fn info<T: fmt::Display>(description: T) -> Result<Logger> {
    print!("{} {}", "INFO".yellow().bold(), description);
    io::stdout().flush()?;
    Ok(Logger {
        start: Instant::now(),
    })
}

impl Logger {
    pub fn end(self) {
        let time_passed = self.start.elapsed();
        let (elapsed, unit) = display_duration(time_passed);
        println!(
            " {} {}{}",
            "took".truecolor(150, 150, 150),
            elapsed.to_string().truecolor(150, 150, 150),
            unit.repr().truecolor(150, 150, 150)
        );
    }
}
