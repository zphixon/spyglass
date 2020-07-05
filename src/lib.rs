//! Provides some basic wrapper types for scoped timers.
//!
//! Because you cannot provide implementations for types outside of your own crate, you need to
//! provide your own timer type that implements the Drop trait and adds it to a lazy static global
//! timer.
//!
//! Example usage:
//!
//! ```
//! # fn main() {}
//! # use inputs::*;
//! use std::time::Instant;
//!
//! #[derive(Debug)]
//! pub struct MyTimer {
//!     name: Option<String>,
//!     begin: Instant,
//! }
//!
//! impl MyTimer {
//!     #[must_use]
//!     pub fn new(name: String) -> Self {
//!         MyTimer {
//!             name: Some(name),
//!             begin: Instant::now(),
//!         }
//!     }
//!
//!     fn end(&mut self) -> Timing {
//!         // MyTimer uses an Option to avoid memory copying overhead. Since
//!         // mem::size_of<String>() is 24, Option::take is just a pointer swap.
//!         Timing {
//!             name: self.name.take().unwrap(),
//!             begin: self.begin,
//!             duration: Instant::now() - self.begin,
//!         }
//!     }
//! }
//!
//! impl Drop for MyTimer {
//!     fn drop(&mut self) {
//!         let end = self.end();
//!
//!         // It's not strictly necessary to spawn a separate thread in order to add the timing to
//!         // the queue, but Drop could block if a lot of MyTimers go out of scope at the same
//!         // time.
//!         std::thread::spawn(move || GLOBAL_TIMER.queue(end));
//!     }
//! }
//!
//! lazy_static::lazy_static! {
//!     static ref GLOBAL_TIMER: Timer = Timer::new();
//! }
//! ```

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Represents a duration of time.
#[derive(Debug)]
pub struct Timing {
    pub name: String,
    pub begin: Instant,
    pub duration: Duration,
}

/// Get the name of the current function.
///
/// Examples:
/// ```
/// # pub fn main() {
/// (|| {
///     mod module {
///         pub trait Trait {
///             fn function(&self) {
///                 let name = inputs::func!();
///                 assert_eq!(
///                     name,
///                     "rust_out::main::{{closure}}::module::Trait::function",
///                 );
///             }
///         }
///         impl Trait for () {}
///     }
///     module::Trait::function(&());
/// })()
/// # }
/// ```
///
/// From Veedrac via [StackOverflow](https://stackoverflow.com/a/40234666).
#[macro_export]
macro_rules! func {
    () => {{
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }

        fn f() {};
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

/// Helper macro to generate a relatively unique name for a timer.
///
/// Examples:
/// ```
/// # use inputs::*;
/// # fn main() {
/// let name = inputs::t!();
/// let name2 = inputs::t!("name2");
///
/// // directory separators will be different depending on your OS
/// assert_eq!("[rust_out::main] src\\lib.rs:5", name);
/// assert_eq!("[rust_out::main] name2 (src\\lib.rs:6)", name2);
/// # }
/// ```
#[macro_export]
macro_rules! t {
    ($name:expr) => {
        format!("[{}] {} ({}:{})", $crate::func!(), $name, file!(), line!())
    };

    () => {
        format!("[{}] {}:{}", $crate::func!(), file!(), line!())
    };
}

/// A timer.
///
/// A timer is essentially just a wrapper around a Mutex. It provides some helper methods to add
/// timers to it asynchronously.
pub struct Timer {
    queue: Mutex<Vec<Timing>>,
}

impl Timer {
    /// Creates a new timer.
    pub fn new() -> Self {
        Timer {
            queue: Mutex::new(Vec::new()),
        }
    }

    /// Adds a timing to the queue.
    ///
    /// The most typical usage is implementing Drop for some type, constructing a Timing, and then
    /// calling this method. It's important to note that this method will block until the Timer can
    /// acquire its mutex, so you may wish to call this method from another thread.
    pub fn queue(&self, timing: Timing) {
        match self.lock() {
            Ok(mut queue) => queue.push(timing),
            Err(e) => eprintln!("couldn't queue \"{}\": {}", timing.name, e),
        }
    }

    /// Locks the timer queue.
    ///
    /// Blocks the current thread until the lock can be obtained.
    pub fn lock(&self) -> Result<MutexGuard<Vec<Timing>>, PoisonError<MutexGuard<Vec<Timing>>>> {
        self.queue.lock()
    }
}

// SAFETY: Timer is just a wrapper around a Mutex.
unsafe impl std::marker::Sync for Timer {}
