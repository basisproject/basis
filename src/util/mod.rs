use ::std::thread;
use ::std::time::Duration;

macro_rules! do_lock {
    ($lock:expr) => {{
        //println!(" >>> lock {} ({}::{})", stringify!($lock), file!(), line!());
        $lock.expect(concat!("factor::util::do_lock!() -- failed to grab lock at ", file!(), "::", line!()))
    }}
}

/// A macro that wraps locking mutexes. Really handy for debugging deadlocks.
#[macro_export]
macro_rules! lock {
    ($lockable:expr) => { do_lock!($lockable.lock()) }
}

/// A macro that wraps read-locking RwLocks. Really handy for debugging
/// deadlocks.
#[macro_export]
macro_rules! lockr {
    ($lockable:expr) => { do_lock!($lockable.read()) }
}

/// A macro that wraps write-locking RwLocks. Really handy for debugging
/// deadlocks.
#[macro_export]
macro_rules! lockw {
    ($lockable:expr) => { do_lock!($lockable.write()) }
}

pub mod logger;
pub mod time;
#[macro_use]
pub mod protobuf;

/// Go to sleeeeep
pub fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

