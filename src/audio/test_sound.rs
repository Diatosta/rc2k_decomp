//! For the audio hooks' self-tests: a DirectSound of the test's. Its object, buffers and
//! 3D buffers note every call (with what their arguments point at) and hand out objects
//! and memory of the test's; each method's result is the test's to choose.

use std::cell::RefCell;

/// A call noted: the object, the method's offset, the arguments (and what they point at).
pub type Noted = (u32, u32, Vec<u32>);

#[derive(Clone, Copy, Default)]
pub struct Results {
    pub create: i32,
    pub duplicate: i32,
    pub query: i32,
    pub lock: i32,
    pub unlock: i32,
    /// The two parts `Lock` hands out, in bytes (the second 0: none).
    pub parts: (u32, u32),
    /// What `GetStatus` gives, and `Play`'s result.
    pub status: u32,
    pub play: i32,
    /// What `GetCurrentPosition` gives as the play cursor, and its result.
    pub cursor: u32,
    pub position: i32,
    /// `GetCaps`' result (0: the caps filled with a pattern).
    pub caps: i32,
    /// What a buffer's `GetVolume` gives, and `SetFormat`'s result.
    pub volume: u32,
    pub format: i32,
    /// A buffer's `Restore` result.
    pub restore: i32,
    /// `SetCooperativeLevel`'s result; the bytes a buffer's `GetCaps` gives (its result is
    /// `caps`).
    pub cooperative: i32,
    pub bytes: u32,
    /// The property set's `QuerySupport`: what it gives and its result; `Set`'s result.
    pub support: u32,
    pub query_support: i32,
    pub set: i32,
}

struct State {
    noted: Vec<Noted>,
    results: Results,
    next_buffer: u32,
}

thread_local!(static STATE: RefCell<State> = const {
    RefCell::new(State { noted: Vec::new(), results: Results { create: 0, duplicate: 0, query: 0, lock: 0, unlock: 0, parts: (0, 0), status: 0, play: 0, cursor: 0, position: 0, caps: 0, volume: 0, format: 0, restore: 0, cooperative: 0, bytes: 0, support: 0, query_support: 0, set: 0 }, next_buffer: 0 })
});

fn note(this: u32, offset: u32, args: Vec<u32>) {
    STATE.with(|s| s.borrow_mut().noted.push((this, offset, args)));
}

fn results() -> Results {
    STATE.with(|s| s.borrow().results)
}

pub struct TestSound {
    sound: Box<[u32; 2]>,
    listener: Box<[u32; 2]>,
    buffers: Box<[[u32; 2]; 4]>,
    three_d: Box<[[u32; 2]; 2]>,
    property_set: Box<[u32; 2]>,
    pub memory: Box<[u8; 0x400]>,
    _tables: Box<[[u32; 0x20]; 5]>,
}

unsafe fn words(at: u32, n: usize) -> Vec<u32> {
    (0..n)
        .map(|i| unsafe { ((at + i as u32 * 4) as *const u32).read_unaligned() })
        .collect()
}

unsafe extern "system" fn create(this: u32, desc: u32, out: u32, outer: u32) -> i32 {
    let mut args = vec![outer];
    unsafe {
        args.extend(words(desc, 5));
        // A primary buffer's description has no format (0).
        let format = ((desc + 0x10) as *const u32).read_unaligned();
        if format != 0 {
            args.extend(words(format, 5));
        }
    }
    note(this, 0x0C, args);
    let r = results().create;
    if r == 0 {
        unsafe { (out as *mut u32).write_unaligned(next_buffer()) };
    }
    r
}

unsafe extern "system" fn duplicate(this: u32, source: u32, out: u32) -> i32 {
    note(this, 0x14, vec![source]);
    let r = results().duplicate;
    if r == 0 {
        unsafe { (out as *mut u32).write_unaligned(next_buffer()) };
    }
    r
}

/// `IKsPropertySet`'s first dword.
const IID_PROPERTY_SET: u32 = 0x31EF_AC30;

unsafe extern "system" fn query(this: u32, iid: u32, out: u32) -> i32 {
    let asked = unsafe { words(iid, 4) };
    let property_set = asked[0] == IID_PROPERTY_SET;
    note(this, 0x00, asked);
    let r = results().query;
    if r == 0 {
        let n = unsafe { (this as *const u32).add(1).read() };
        let object = if property_set {
            PROPERTY_SET.with(|p| *p.borrow())
        } else {
            THREE_D.with(|t| t.borrow()[n as usize % 2])
        };
        unsafe { (out as *mut u32).write_unaligned(object) };
    }
    r
}

/// `IKsPropertySet::Set`: the set's first dword, the id, the instance, the data's dwords.
#[allow(clippy::too_many_arguments)] // the method's own arguments
unsafe extern "system" fn property_set(
    this: u32,
    set: u32,
    id: u32,
    instance: u32,
    instance_bytes: u32,
    data: u32,
    bytes: u32,
) -> i32 {
    let mut args = vec![
        unsafe { words(set, 1)[0] },
        id,
        instance,
        instance_bytes,
        bytes,
    ];
    args.extend(unsafe { words(data, bytes as usize / 4) });
    note(this, 0x10, args);
    results().set
}

unsafe extern "system" fn query_support(this: u32, set: u32, id: u32, support: u32) -> i32 {
    note(this, 0x14, vec![unsafe { words(set, 1)[0] }, id]);
    let r = results();
    if r.query_support == 0 {
        unsafe { (support as *mut u32).write_unaligned(r.support) };
    }
    r.query_support
}

#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn lock(
    this: u32,
    offset: u32,
    bytes: u32,
    p1: u32,
    n1: u32,
    p2: u32,
    n2: u32,
    flags: u32,
) -> i32 {
    note(this, 0x2C, vec![offset, bytes, flags]);
    let r = results();
    if r.lock == 0 {
        let memory = MEMORY.with(|m| *m.borrow());
        unsafe {
            (p1 as *mut u32).write_unaligned(memory);
            (n1 as *mut u32).write_unaligned(r.parts.0);
            (p2 as *mut u32).write_unaligned(if r.parts.1 == 0 { 0 } else { memory + 0x200 });
            (n2 as *mut u32).write_unaligned(r.parts.1);
        }
    }
    r.lock
}

unsafe extern "system" fn unlock(this: u32, p1: u32, n1: u32, p2: u32, n2: u32) -> i32 {
    note(this, 0x4C, vec![p1, n1, p2, n2]);
    results().unlock
}

unsafe extern "system" fn current_position(this: u32, play: u32, write: u32) -> i32 {
    note(this, 0x10, vec![write]);
    let r = results();
    if r.position == 0 {
        unsafe { (play as *mut u32).write_unaligned(r.cursor) };
    }
    r.position
}
unsafe extern "system" fn get_status(this: u32, status: u32) -> i32 {
    note(this, 0x24, vec![]);
    unsafe { (status as *mut u32).write_unaligned(results().status) };
    0
}
unsafe extern "system" fn play(this: u32, a: u32, b: u32, flags: u32) -> i32 {
    note(this, 0x30, vec![a, b, flags]);
    results().play
}
unsafe extern "system" fn set_position(this: u32, a: u32) -> i32 {
    note(this, 0x34, vec![a]);
    0
}
unsafe extern "system" fn stop(this: u32) -> i32 {
    note(this, 0x48, vec![]);
    0
}
unsafe extern "system" fn get_caps(this: u32, caps: u32) -> i32 {
    note(this, 0x10, unsafe { words(caps, 1) });
    let r = results().caps;
    if r == 0 {
        for n in 1..0x18u32 {
            unsafe { ((caps + n * 4) as *mut u32).write_unaligned(0x1000 + n * 0x11) };
        }
    }
    r
}
unsafe extern "system" fn get_volume(this: u32, volume: u32) -> i32 {
    note(this, 0x18, vec![]);
    unsafe { (volume as *mut u32).write_unaligned(results().volume) };
    0
}
unsafe extern "system" fn set_format(this: u32, format: u32) -> i32 {
    note(this, 0x38, unsafe { words(format, 5) });
    results().format
}
unsafe extern "system" fn restore(this: u32) -> i32 {
    note(this, 0x50, vec![]);
    results().restore
}
unsafe extern "system" fn cooperative_level(this: u32, window: u32, level: u32) -> i32 {
    note(this, 0x18, vec![window, level]);
    results().cooperative
}
unsafe extern "system" fn buffer_caps(this: u32, caps: u32) -> i32 {
    note(this, 0x0C, unsafe { words(caps, 1) });
    let r = results();
    if r.caps == 0 {
        unsafe { ((caps + 8) as *mut u32).write_unaligned(r.bytes) };
    }
    r.caps
}
unsafe extern "system" fn release(this: u32) -> i32 {
    note(this, 0x08, vec![]);
    0
}
unsafe extern "system" fn commit(this: u32) -> i32 {
    note(this, 0x44, vec![]);
    0
}
unsafe extern "system" fn one(this: u32, a: u32) -> i32 {
    note(this, 0x3C, vec![a]);
    0
}
unsafe extern "system" fn pan(this: u32, a: u32) -> i32 {
    note(this, 0x40, vec![a]);
    0
}
unsafe extern "system" fn frequency(this: u32, a: u32) -> i32 {
    note(this, 0x44, vec![a]);
    0
}
unsafe extern "system" fn min_distance(this: u32, a: u32, b: u32) -> i32 {
    note(this, 0x44, vec![a, b]);
    0
}
unsafe extern "system" fn position(this: u32, a: u32, b: u32, c: u32, d: u32) -> i32 {
    note(this, 0x4C, vec![a, b, c, d]);
    0
}

thread_local!(static BUFFERS: RefCell<[u32; 4]> = const { RefCell::new([0; 4]) });
thread_local!(static THREE_D: RefCell<[u32; 2]> = const { RefCell::new([0; 2]) });
thread_local!(static PROPERTY_SET: RefCell<u32> = const { RefCell::new(0) });
thread_local!(static MEMORY: RefCell<u32> = const { RefCell::new(0) });

fn next_buffer() -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let n = s.next_buffer as usize % 4;
        s.next_buffer += 1;
        BUFFERS.with(|b| b.borrow()[n])
    })
}

impl Default for TestSound {
    fn default() -> Self {
        Self::new()
    }
}

impl TestSound {
    pub fn new() -> Self {
        let mut tables = Box::new([[0u32; 0x20]; 5]);
        tables[0][0x0C / 4] = create as *const () as u32;
        tables[0][0x14 / 4] = duplicate as *const () as u32;
        tables[0][0x08 / 4] = release as *const () as u32;
        tables[0][0x10 / 4] = get_caps as *const () as u32;
        tables[0][0x18 / 4] = cooperative_level as *const () as u32;
        tables[1][0] = query as *const () as u32;
        tables[1][0x08 / 4] = release as *const () as u32;
        tables[1][0x18 / 4] = get_volume as *const () as u32;
        tables[1][0x38 / 4] = set_format as *const () as u32;
        tables[1][0x50 / 4] = restore as *const () as u32;
        tables[1][0x0C / 4] = buffer_caps as *const () as u32;
        tables[1][0x2C / 4] = lock as *const () as u32;
        tables[1][0x3C / 4] = one as *const () as u32;
        tables[1][0x40 / 4] = pan as *const () as u32;
        tables[1][0x44 / 4] = frequency as *const () as u32;
        tables[1][0x4C / 4] = unlock as *const () as u32;
        tables[1][0x24 / 4] = get_status as *const () as u32;
        tables[1][0x10 / 4] = current_position as *const () as u32;
        tables[1][0x30 / 4] = play as *const () as u32;
        tables[1][0x34 / 4] = set_position as *const () as u32;
        tables[1][0x48 / 4] = stop as *const () as u32;
        tables[3][0x44 / 4] = commit as *const () as u32;
        tables[2][0] = query as *const () as u32;
        tables[2][0x44 / 4] = min_distance as *const () as u32;
        tables[4][0x10 / 4] = property_set as *const () as u32;
        tables[4][0x14 / 4] = query_support as *const () as u32;
        tables[2][0x4C / 4] = position as *const () as u32;
        let t = |n: usize| tables[n].as_ptr() as u32;
        let sound = Box::new([t(0), 0]);
        let listener = Box::new([t(3), 0]);
        let buffers = Box::new([[t(1), 0], [t(1), 1], [t(1), 2], [t(1), 3]]);
        let three_d = Box::new([[t(2), 0], [t(2), 1]]);
        let property_set = Box::new([t(4), 0]);
        let mut memory = Box::new([0u8; 0x400]);
        BUFFERS.with(|b| *b.borrow_mut() = [0, 1, 2, 3].map(|n| buffers[n].as_ptr() as u32));
        THREE_D.with(|b| *b.borrow_mut() = [0, 1].map(|n| three_d[n].as_ptr() as u32));
        MEMORY.with(|m| *m.borrow_mut() = memory.as_mut_ptr() as u32);
        PROPERTY_SET.with(|p| *p.borrow_mut() = property_set.as_ptr() as u32);
        Self {
            sound,
            listener,
            buffers,
            three_d,
            property_set,
            memory,
            _tables: tables,
        }
    }

    pub fn sound(&self) -> u32 {
        self.sound.as_ptr() as u32
    }

    /// A 3D listener: `CommitDeferredSettings` (+0x44).
    pub fn listener(&self) -> u32 {
        self.listener.as_ptr() as u32
    }

    pub fn buffer(&self, n: usize) -> u32 {
        self.buffers[n].as_ptr() as u32
    }

    pub fn three_d(&self, n: usize) -> u32 {
        self.three_d[n].as_ptr() as u32
    }

    /// An `IKsPropertySet` (what a 3D buffer hands out for it): `Set` (+0x10),
    /// `QuerySupport` (+0x14).
    pub fn property_set(&self) -> u32 {
        self.property_set.as_ptr() as u32
    }

    /// Begins a run: the results chosen, the notes and the handing out started anew.
    pub fn begin(&self, results: Results) {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.noted.clear();
            s.results = results;
            s.next_buffer = 1;
        });
    }

    pub fn noted(&self) -> Vec<Noted> {
        STATE.with(|s| std::mem::take(&mut s.borrow_mut().noted))
    }
}
