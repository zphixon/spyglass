use spyglass::{func, t, Timer, Timing};

use std::time::{Duration, Instant};

fn main() {
    {
        let _a = MyTimer::new(t!("maggie"));
        let _b = MyTimer::new(t!("milly"));
        x::something_expensive();
        let _c = MyTimer::new(t!("molly"));
        let _d = MyTimer::new(t!("may"));
    }

    {
        let _t = MyTimer::new(t!());
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

    pub fn something_expensive() {
        let _x = MyTimer::new(func!());

        for _ in 0..15 {
            dot();
        }

        println!();
    }

    fn dot() {
        use std::io::Write;
        print!(".");
        std::io::stdout().flush().unwrap();
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[derive(Debug)]
pub struct MyTimer {
    name: String,
    begin: Instant,
}

impl MyTimer {
    #[must_use]
    pub fn new<T: ToString>(name: T) -> Self {
        MyTimer {
            name: name.to_string(),
            begin: Instant::now(),
        }
    }

    fn end(&mut self) -> Timing {
        Timing {
            name: self.name.clone(),
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

spyglass::lazy_static! {
    static ref GLOBAL_TIMER: Timer = Timer::new();
}
