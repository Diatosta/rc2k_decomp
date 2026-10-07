use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, game_addr, hook_asm, log};
use crate::strings::utils::bytes_str;
use crate::threading::find_thread_slot;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "files::resolve_path",
    0x403206 - 0x400000,
    thunk,
    &TRAMPOLINE,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE
);

const MEDIA_MODE_RVA: usize = 0x5189A4 - 0x400000;
const BASE_A_RVA: usize = 0x5189A8 - 0x400000;
const BASE_B_RVA: usize = 0x5295B4 - 0x400000;
const PCT_RVA: usize = 0x518AA8 - 0x400000;
const GLOBAL_BUF_RVA: usize = 0x94E9B0 - 0x400000;

const BUF_LEN: usize = 0x100; // ThreadSlot + 0x10 .. + 0x110

fn at(p: &[u8], i: usize) -> u8 {
    p.get(i).copied().unwrap_or(0)
}

/// None = path is returned unchanged. `path` excludes the NUL.
pub fn resolve(
    path: &[u8],
    media_mode: u32,
    base_a: &[u8],
    base_b: &[u8],
    pct: &[u8],
) -> Option<Vec<u8>> {
    if path.starts_with(b"var\\") || path.starts_with(b"save") || at(path, 1) == b':' {
        return None;
    }
    let (semi3, semi4) = (path.starts_with(b";3"), path.starts_with(b";4"));
    let use_b = !semi4 && (media_mode >= 3 || (!semi3 && media_mode == 2)); // unsigned compares
    let base = if use_b { base_b } else { base_a };
    let rel = if at(path, 0) == b';' {
        &path[2.min(path.len())..]
    } else {
        path
    };

    let mut out = base.to_vec();
    for &c in rel {
        if c == b'%' {
            out.extend_from_slice(pct)
        } else {
            out.push(c)
        }
    }
    out.push(0);
    Some(out)
}

unsafe fn cstr<'a>(addr: usize) -> &'a [u8] {
    unsafe { CStr::from_ptr(addr as *const c_char) }.to_bytes()
}

struct Globals<'a> {
    mode: u32,
    a: &'a [u8],
    b: &'a [u8],
    pct: &'a [u8],
}
unsafe fn globals<'a>() -> Globals<'a> {
    unsafe {
        Globals {
            mode: *(game_addr(MEDIA_MODE_RVA) as *const u32),
            a: cstr(game_addr(BASE_A_RVA)),
            b: cstr(game_addr(BASE_B_RVA)),
            pct: cstr(game_addr(PCT_RVA)),
        }
    }
}

/// slot+0x10 if the current thread has a ThreadSlot, else the global buffer.
fn out_buffer() -> *mut u8 {
    let slot = unsafe { find_thread_slot::reimplemented() };
    if slot.is_null() {
        game_addr(GLOBAL_BUF_RVA) as *mut u8
    } else {
        unsafe { (slot as *mut u8).add(0x10) }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let path = unsafe { cstr(r.eax as usize) };
    let g = unsafe { globals() };
    let mine = resolve(path, g.mode, g.a, g.b, g.pct);

    // The original overflows the buffer on a long path: let it do exactly that.
    if mine.as_ref().is_some_and(|m| m.len() > BUF_LEN) {
        unsafe { call_original(r) };
        return;
    }
    if !SHADOW_MODE.load(Ordering::Relaxed) {
        if let Some(m) = mine {
            let buf = out_buffer();
            unsafe { std::ptr::copy_nonoverlapping(m.as_ptr(), buf, m.len()) };
            r.eax = buf as u32;
        }
        return; // None: eax unchanged
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let ok = match &mine {
        None => o.eax == r.eax,
        Some(m) => {
            o.eax == out_buffer() as u32
                && unsafe { std::slice::from_raw_parts(o.eax as *const u8, m.len()) } == &m[..]
        }
    };
    if !ok && HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed) < MAX_LOGGED {
        log(&format!("resolve_path MISMATCH path={}", bytes_str(path)));
    }
    r.eax = o.eax;
}

pub fn self_test() -> (usize, usize) {
    let paths: [&[u8]; 11] = [
        b"var\\logfile.txt\0",
        b"save1.dat\0",
        b"C:\\x.txt\0",
        b"data\\a.txt\0",
        b";3data\\a.txt\0",
        b";4data\\a.txt\0",
        b";5x\0",
        b"d\\%\\f.txt\0",
        b"%%\0",
        b";3%x\0",
        b"x\0",
    ];
    let g = unsafe { globals() };
    let buf = out_buffer();
    let (mut cases, mut fails) = (0usize, 0usize);
    for p in paths {
        let snapshot: Vec<u8> = unsafe { std::slice::from_raw_parts(buf, BUF_LEN) }.to_vec();
        let mut r = Regs {
            edi: 0x1111_1111,
            esi: 0x2222_2222,
            ebp: 0x3333_3333,
            esp: 0,
            ebx: 0x4444_4444,
            edx: 0x5555_5555,
            ecx: 0x6666_6666,
            eax: p.as_ptr() as u32,
        };
        unsafe { call_original(&mut r) };

        let mine = resolve(&p[..p.len() - 1], g.mode, g.a, g.b, g.pct);
        let now = unsafe { std::slice::from_raw_parts(buf, BUF_LEN) };
        let mut want = snapshot.clone();
        let eax_want = match &mine {
            None => p.as_ptr() as u32,
            Some(m) => {
                want[..m.len()].copy_from_slice(m);
                buf as u32
            }
        };
        let preserved = r.ebx == 0x4444_4444
            && r.ecx == 0x6666_6666
            && r.edx == 0x5555_5555
            && r.esi == 0x2222_2222
            && r.edi == 0x1111_1111
            && r.ebp == 0x3333_3333;
        cases += 1;
        if r.eax != eax_want || now != &want[..] || !preserved {
            fails += 1;
            log(&format!(
                "resolve_path FAIL path={} eax={:#x} want {:#x} preserved={preserved}",
                bytes_str(&p[..p.len() - 1]),
                r.eax,
                eax_want
            ));
        }
        unsafe { std::ptr::copy_nonoverlapping(snapshot.as_ptr(), buf, BUF_LEN) }; // restore game memory
    }
    (cases, fails)
}
