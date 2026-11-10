//! The game's threads, events and mutexes on std (they
//! were Win32's `CreateThread`, `CreateEventA`, `CreateMutexA`, `WaitForSingleObject`,
//! `WaitForMultipleObjects`, `SetEvent`, `ReleaseMutex`, `CloseHandle`, `ExitThread`,
//! `GetCurrentThreadId`, `SetThreadPriority`).
//!
//! The game keeps a handle as a 32-bit number in its memory: here a handle is a key in a
//! table of Rust objects, each a state under a `Mutex` with a `Condvar` for its waiters.
//! The numbers are never 0 or 1 (the game tests a handle with `cmp eax, 1`). Thread ids are
//! numbers given by this module, 1 up, one per thread for its life; a slot's thread gets
//! its id before it starts, as `CreateThread` wrote it into the slot.

#![allow(unsafe_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// WaitForSingleObject's answers.
pub const WAIT_OBJECT_0: u32 = 0;
pub const WAIT_TIMEOUT: u32 = 0x102;
pub const WAIT_FAILED: u32 = u32::MAX;
/// A wait with no end.
pub const INFINITE: u32 = u32::MAX;

#[derive(Debug)]
enum State {
    Event { manual: bool, set: bool },
    Mutex { owner: u32, count: u32 },
    Thread { done: bool },
}

struct Object {
    state: Mutex<State>,
    changed: Condvar,
}

impl Object {
    fn new(state: State) -> Arc<Object> {
        Arc::new(Object {
            state: Mutex::new(state),
            changed: Condvar::new(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The handles: 0x7000_0000 up in fours (clear of 0 and 1, and of the system's own).
static OBJECTS: Mutex<Option<HashMap<u32, Arc<Object>>>> = Mutex::new(None);
static NEXT_HANDLE: AtomicU32 = AtomicU32::new(0x7000_0000);

fn add(object: Arc<Object>) -> u32 {
    let handle = NEXT_HANDLE.fetch_add(4, Ordering::Relaxed);
    OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .insert(handle, object);
    handle
}

fn object(handle: u32) -> Option<Arc<Object>> {
    OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .get(&handle)
        .cloned()
}

// ---- thread ids ----

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static ID: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// Whether `spawn` made the thread (`exit_thread` unwinds only there).
    static SPAWNED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// An id for a thread `spawn` will start (a slot gets it before its thread runs).
pub fn new_thread_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// The running thread's id (`GetCurrentThreadId`): given at its first asking, or before it
/// started for a thread `spawn` made.
pub fn thread_id() -> u32 {
    ID.with(|id| {
        if id.get() == 0 {
            id.set(NEXT_ID.fetch_add(1, Ordering::Relaxed));
        }
        id.get()
    })
}

// ---- events and mutexes ----

/// An event (`CreateEventA` with no name): its handle.
pub fn create_event(manual: bool, set: bool) -> u32 {
    add(Object::new(State::Event { manual, set }))
}

/// A mutex owned by no one (`CreateMutexA(0, FALSE, 0)`): its handle.
pub fn create_mutex() -> u32 {
    add(Object::new(State::Mutex { owner: 0, count: 0 }))
}

/// The event set and its waiters woken (`SetEvent`); false if `handle` is no event.
pub fn set_event(handle: u32) -> bool {
    let Some(o) = object(handle) else {
        return false;
    };
    let mut state = o.lock();
    let State::Event { set, .. } = &mut *state else {
        return false;
    };
    *set = true;
    o.changed.notify_all();
    true
}

/// The mutex let go once by its owner (`ReleaseMutex`); false if `handle` is no mutex or
/// the running thread does not own it.
pub fn release_mutex(handle: u32) -> bool {
    let Some(o) = object(handle) else {
        return false;
    };
    let mut state = o.lock();
    let State::Mutex { owner, count } = &mut *state else {
        return false;
    };
    if *owner != thread_id() || *count == 0 {
        return false;
    }
    *count -= 1;
    if *count == 0 {
        *owner = 0;
        o.changed.notify_all();
    }
    true
}

/// Whether the object lets the waiter through now; if so it is taken (an auto-reset
/// event cleared, a mutex owned once more).
fn take(state: &mut State, me: u32) -> bool {
    match state {
        State::Event { manual, set } => {
            let through = *set;
            if through && !*manual {
                *set = false;
            }
            through
        }
        State::Mutex { owner, count } => {
            if *owner == 0 || *owner == me {
                *owner = me;
                *count += 1;
                true
            } else {
                false
            }
        }
        State::Thread { done } => *done,
    }
}

/// When a wait of `milliseconds` from now ends (None: never).
fn deadline(milliseconds: u32) -> Option<Instant> {
    (milliseconds != INFINITE).then(|| Instant::now() + Duration::from_millis(milliseconds as u64))
}

/// Waits on the object until it lets the thread through or the deadline passes.
fn wait_on(o: &Object, until: Option<Instant>) -> u32 {
    let me = thread_id();
    let mut state = o.lock();
    loop {
        if take(&mut state, me) {
            return WAIT_OBJECT_0;
        }
        state = match until {
            None => o
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            Some(until) => {
                let now = Instant::now();
                if now >= until {
                    return WAIT_TIMEOUT;
                }
                o.changed
                    .wait_timeout(state, until - now)
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .0
            }
        };
    }
}

/// `WaitForSingleObject`: WAIT_OBJECT_0 when let through, WAIT_TIMEOUT, or WAIT_FAILED for
/// a handle that is none.
pub fn wait(handle: u32, milliseconds: u32) -> u32 {
    match object(handle) {
        Some(o) => wait_on(&o, deadline(milliseconds)),
        None => WAIT_FAILED,
    }
}

/// `WaitForMultipleObjects` waiting for all, on threads (the one use: the shutdown's):
/// WAIT_OBJECT_0 when all have ended, WAIT_TIMEOUT, or WAIT_FAILED when a handle is not a
/// thread's.
pub fn wait_all_threads(handles: &[u32], milliseconds: u32) -> u32 {
    let until = deadline(milliseconds);
    let mut objects = Vec::new();
    for &h in handles {
        match object(h) {
            Some(o) if matches!(*o.lock(), State::Thread { .. }) => objects.push(o),
            _ => return WAIT_FAILED,
        }
    }
    for o in objects {
        if wait_on(&o, until) == WAIT_TIMEOUT {
            return WAIT_TIMEOUT;
        }
    }
    WAIT_OBJECT_0
}

/// The handle given back (`CloseHandle`); false if it is none. A thread runs on.
pub fn close(handle: u32) -> bool {
    OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .remove(&handle)
        .is_some()
}

// ---- threads ----

/// The marker `exit_thread` unwinds with.
struct ThreadExit;

/// A thread started on `routine` with the id `id` (`new_thread_id`; `CreateThread`): its
/// handle. A routine that panics ends the program (as a fault in a Win32 thread did); None
/// if the system starts no thread.
pub fn spawn(name: &str, id: u32, routine: impl FnOnce() + Send + 'static) -> Option<u32> {
    let o = Object::new(State::Thread { done: false });
    let mine = o.clone();
    std::thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || {
            ID.with(|cell| cell.set(id));
            SPAWNED.with(|cell| cell.set(true));
            let ended = std::panic::catch_unwind(std::panic::AssertUnwindSafe(routine));
            *mine.lock() = State::Thread { done: true };
            mine.changed.notify_all();
            if let Err(payload) = ended
                && !payload.is::<ThreadExit>()
            {
                crate::standalone::finish(
                    &format!(
                        "sync: the thread {id} ({:?}) panicked",
                        std::thread::current().name()
                    ),
                    1,
                );
            }
        })
        .ok()?;
    Some(add(o))
}

/// Ends the running thread (`ExitThread`), from any depth of a thread `spawn` made: it
/// unwinds to the thread's start. On another thread it ends the program.
pub fn exit_thread() -> ! {
    if !SPAWNED.with(std::cell::Cell::get) {
        crate::standalone::finish("sync: a thread not started here was to end itself", 1);
    }
    std::panic::resume_unwind(Box::new(ThreadExit))
}

/// The running thread's priority (`SetThreadPriority`'s values: 0 normal, 1 .. 2 above,
/// 15 time-critical, below 0 lower), through SDL.
pub fn set_priority(priority: i32) {
    use sdl3_sys::thread::*;
    let p = match priority {
        15.. => SDL_THREAD_PRIORITY_TIME_CRITICAL,
        1..=14 => SDL_THREAD_PRIORITY_HIGH,
        0 => SDL_THREAD_PRIORITY_NORMAL,
        _ => SDL_THREAD_PRIORITY_LOW,
    };
    // SAFETY: a plain value; it fails harmlessly where the system refuses.
    unsafe { SDL_SetCurrentThreadPriority(p) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_wake_and_reset() {
        let auto = create_event(false, false);
        assert_eq!(wait(auto, 10), WAIT_TIMEOUT);
        assert!(set_event(auto));
        assert_eq!(wait(auto, 0), WAIT_OBJECT_0);
        assert_eq!(wait(auto, 0), WAIT_TIMEOUT, "an auto-reset event clears");
        let manual = create_event(true, true);
        assert_eq!(wait(manual, 0), WAIT_OBJECT_0);
        assert_eq!(wait(manual, 0), WAIT_OBJECT_0);
        let h = spawn("test", new_thread_id(), move || {
            std::thread::sleep(Duration::from_millis(20));
            set_event(auto);
        })
        .unwrap();
        assert_eq!(wait(auto, 5000), WAIT_OBJECT_0);
        assert_eq!(wait_all_threads(&[h], 5000), WAIT_OBJECT_0);
        assert!(close(auto) && close(manual) && close(h));
        assert_eq!(wait(auto, 0), WAIT_FAILED);
    }

    #[test]
    fn a_mutex_is_owned_by_one_thread() {
        let m = create_mutex();
        assert_eq!(wait(m, 0), WAIT_OBJECT_0);
        assert_eq!(wait(m, 0), WAIT_OBJECT_0, "its owner again");
        let h = spawn("test", new_thread_id(), move || {
            assert_eq!(wait(m, 0), WAIT_TIMEOUT);
            assert!(!release_mutex(m));
        })
        .unwrap();
        assert_eq!(wait_all_threads(&[h], 5000), WAIT_OBJECT_0);
        assert!(release_mutex(m) && release_mutex(m) && !release_mutex(m));
        close(m);
    }

    #[test]
    fn threads_have_their_ids_and_exit_from_any_depth() {
        let seen = Arc::new(AtomicU32::new(0));
        let s = seen.clone();
        let id = new_thread_id();
        let h = spawn("test", id, move || {
            s.store(thread_id(), Ordering::SeqCst);
            exit_thread();
        })
        .unwrap();
        assert_eq!(wait(h, 5000), WAIT_OBJECT_0);
        assert_eq!(seen.load(Ordering::SeqCst), id);
        assert_ne!(id, thread_id());
        close(h);
    }
}
