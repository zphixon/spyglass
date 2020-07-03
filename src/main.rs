fn main() {
    {
        let _ = Timer::new("thread 1");
        let _ = Timer::new("thread 2");
        let _ = Timer::new("thread 3");
        let _ = Timer::new("thread 4");
    }

    while let Ok(timing) = GLOBAL_TIMER.receiver.recv() {
        println!("{:?}", timing);
    }
}

use lazy_static;

use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Timing {
    name: Option<String>,
    duration: Duration,
}

#[derive(Debug)]
struct Timer {
    name: Option<String>,
    begin: Instant,
}

unsafe impl std::marker::Sync for Timer {}

impl Timer {
    pub fn new(name: &str) -> Self {
        Timer {
            name: Some(name.to_owned()),
            begin: std::time::Instant::now(),
        }
    }

    pub fn end(&mut self) -> Timing {
        Timing {
            // use Option::take to avoid cloning
            name: self.name.take(),
            duration: std::time::Instant::now() - self.begin,
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // or use RwLock and spawn a new thread to drop ourselves
        let sender = GLOBAL_TIMER.sender.clone();
        match sender.send(self.end()) {
            Ok(_) => {}
            Err(_) => eprintln!("couldn't drop {:?}", self.name),
        }
    }
}

struct GlobalTimer {
    pub sender: Sender<Timing>,
    pub receiver: Receiver<Timing>,
}

unsafe impl std::marker::Sync for GlobalTimer {}

lazy_static::lazy_static! {
    static ref GLOBAL_TIMER: GlobalTimer = {
        let (sender, receiver) = std::sync::mpsc::channel();
        GlobalTimer {
            sender,
            receiver,
        }
    };
}
