use inputs::{tn, Timer, Timing};

use std::time::{Duration, Instant};

fn main() {
    {
        let _a = MyTimer::new(tn!("maggie"));
        let _b = MyTimer::new(tn!("milly"));
        x::something();
        let _c = MyTimer::new(tn!("molly"));
        let _d = MyTimer::new(tn!("may"));
    }

    // wait for all the drop threads to finish
    std::thread::sleep(Duration::from_micros(1));

    match GLOBAL_TIMER.lock() {
        Ok(queue) => {
            for timing in queue.iter() {
                println!(
                    "{} took {}s ({}ns)",
                    timing.name,
                    timing.duration.as_secs(),
                    timing.duration.as_nanos()
                );
            }
        }

        Err(e) => eprintln!("couldn't check timings: {}", e),
    }
}

mod x {
    use super::*;
    pub fn something() {
        let _x = MyTimer::new(tn!("something"));
        std::thread::sleep(Duration::from_secs(14));
    }
}

#[derive(Debug)]
pub struct MyTimer {
    name: Option<String>,
    begin: Instant,
}

impl MyTimer {
    #[must_use]
    pub fn new(name: String) -> Self {
        MyTimer {
            name: Some(name),
            begin: Instant::now(),
        }
    }

    fn end(&mut self) -> Timing {
        Timing {
            name: self.name.take().unwrap(),
            begin: self.begin,
            duration: Instant::now() - self.begin,
        }
    }
}

impl Drop for MyTimer {
    fn drop(&mut self) {
        let end = self.end();
        std::thread::spawn(move || GLOBAL_TIMER.queue(end));
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_TIMER: Timer = Timer::new();
}
