//! The two stream commands (converted 2026-10-21 from `audio::stream_command` 0x40C194,
//! command 9, the sound stream, and `audio::music_command` 0x40C4DD, command 0xB, the
//! music). Actions (unsigned): 0 stop; 1 the thread's turn (feed while playing; stopped
//! after it: stop, and for the music the next tune); 3 the volume of what plays; any other
//! plays (stopped first if playing; nothing for a volume <= 0 or, the music, with no
//! music; a stop for no path or tune).
//!
//! Music tunes: the file is the music path with the tune's two last digits; tune 0xA is
//! the tune turn plus 10, for the first turn (counted on, 0 .. 5) whose file exists. When a tune
//! stops the next tune is played, never cleared there: the second tune repeats. Fixed
//! (2026-10-20, notes' section 16): a turn past 5 starts from 0, and after `RETRIES` plays
//! in a row that stop at their first feed the music stops (the original: both for ever,
//! the audio thread hung).

/// A stream player and what it is told.
pub trait Player {
    type Error;
    fn playing(&mut self) -> bool;
    /// The volume kept in the player.
    fn keep_volume(&mut self, volume: u32);
    /// Started on its file: false when it failed.
    fn start(&mut self) -> Result<bool, Self::Error>;
    fn feed(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    /// The kept volume given to its buffer.
    fn apply_volume(&mut self) -> Result<(), Self::Error>;
}

/// Command 9 (`has_path`: the sender gave a file).
pub fn stream_command<P: Player>(
    p: &mut P,
    has_path: bool,
    volume: i32,
    action: u32,
) -> Result<(), P::Error> {
    match action {
        0 => return p.stop(),
        1 => {
            if !p.playing() {
                return Ok(());
            }
        }
        3 => {
            if p.playing() {
                p.keep_volume(volume as u32);
                p.apply_volume()?;
            }
            return Ok(());
        }
        _ => {
            if p.playing() {
                p.stop()?;
            }
            if volume <= 0 {
                return Ok(());
            }
            if !has_path {
                return p.stop();
            }
            p.keep_volume(volume as u32);
            if !p.start()? {
                return p.stop();
            }
        }
    }
    p.feed()?;
    if !p.playing() {
        p.stop()?;
    }
    Ok(())
}

/// The music's player: its file named by a tune, and whether the file is there.
pub trait Tunes: Player {
    /// The music path's two digits made `n`'s.
    fn name(&mut self, n: u32);
    fn exists(&mut self) -> Result<bool, Self::Error>;
    /// The fix's stop (`RETRIES` plays stopped at once), for the log.
    fn gave_up(&mut self, tune: u32);
}

/// The music's variables (`g_nextTune`, `g_noMusic`, `g_tuneTurn`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Music {
    pub next_tune: u32,
    pub no_music: u32,
    pub turn: u32,
}

/// The plays in a row that may stop at their first feed: the tune, then tune 0xA's six
/// turns.
pub const RETRIES: u32 = 7;

/// Command 0xB: `tune`, with `next` to follow.
pub fn music_command<P: Tunes>(
    p: &mut P,
    m: &mut Music,
    tune: u32,
    next: u32,
    volume: i32,
    action: u32,
) -> Result<(), P::Error> {
    let stop_and_forget = |p: &mut P, m: &mut Music| -> Result<(), P::Error> {
        p.stop()?;
        m.next_tune = 0;
        Ok(())
    };
    let mut tune = tune;
    let mut feed_first = false;
    match action {
        0 => return stop_and_forget(p, m),
        1 => {
            if !p.playing() {
                return Ok(());
            }
            feed_first = true;
        }
        3 => {
            if p.playing() {
                p.keep_volume(volume as u32);
                p.apply_volume()?;
            }
            return Ok(());
        }
        _ => {
            if p.playing() {
                p.stop()?;
            }
            if volume <= 0 || m.no_music != 0 {
                return Ok(());
            }
            if tune == 0 {
                return stop_and_forget(p, m);
            }
            m.next_tune = next;
            p.keep_volume(volume as u32);
        }
    }
    let mut plays = 0;
    loop {
        if !feed_first {
            plays += 1;
            if plays > RETRIES {
                p.gave_up(tune);
                return stop_and_forget(p, m);
            }
            if tune == 0xA {
                if m.turn >= 6 {
                    m.turn = 0;
                }
                let first = m.turn;
                loop {
                    let n = 0xA + m.turn;
                    m.turn = if m.turn.wrapping_add(1) >= 6 {
                        0
                    } else {
                        m.turn + 1
                    };
                    p.name(n);
                    if p.exists()? {
                        break;
                    }
                    if first == m.turn {
                        m.next_tune = 0;
                        return Ok(());
                    }
                }
            } else {
                p.name(tune);
            }
            if !p.start()? {
                return stop_and_forget(p, m);
            }
        }
        feed_first = false;
        p.feed()?;
        if p.playing() {
            return Ok(());
        }
        p.stop()?;
        tune = m.next_tune;
        if tune == 0 {
            return stop_and_forget(p, m);
        }
    }
}

/// What a looping buffer's play cursor moved over since the last turn (from `old` to
/// `new`, round a buffer of `size`), at most `most`: None when it has not moved; else the
/// bytes, and when they were cut the write place put that far ahead of the cursor. The
/// streams' (`audio::feed_stream`) and the mixer's (`audio::mix` 0x40BDC5) rule.
pub fn cursor_moved(old: u32, new: u32, size: u32, most: u32) -> Option<(u32, Option<u32>)> {
    let mut moved = new.wrapping_sub(old);
    if (moved as i32) < 0 {
        moved = moved.wrapping_add(size);
    }
    if moved == 0 {
        return None;
    }
    if moved > most {
        return Some((most, Some(round(new.wrapping_add(most), size))));
    }
    Some((moved, None))
}

/// A place round a buffer of `size` (one wrap at most).
pub fn round(at: u32, size: u32) -> u32 {
    if at >= size {
        at.wrapping_sub(size)
    } else {
        at
    }
}

/// A stream player (`g_soundStream` 0x506960, `g_musicStream` 0x506990): a WAV file
/// played through a looping buffer refilled each turn (converted 2026-10-21 from
/// `audio::feed_stream` 0x40C349 and `audio::fill_stream_part` 0x40C42C).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioStream {
    pub playing: u32,
    /// An index into `g_volumeTable`.
    pub volume: u32,
    pub buffer: u32,
    /// 1: `file` is open.
    pub open: u32,
    pub file: u32,
    pub play_cursor: u32,
    /// Where the next bytes go in the buffer.
    pub write_at: u32,
    pub format: u32,
    /// The buffer's size (0x10000 in both players).
    pub bytes: u32,
    /// The most a turn writes, and how far the write place starts ahead (0x8000).
    pub step: u32,
    /// The silence still allowed after the file's end.
    pub tail: i32,
}

const _: () = assert!(std::mem::size_of::<AudioStream>() == 0x2C);

impl AudioStream {
    /// The turn's bytes to write, from the play cursor's new place: what it moved over
    /// (round the buffer), at most `step` (the write place then put that far ahead of the
    /// cursor). None: it has not moved.
    pub fn moved(&mut self, cursor: u32) -> Option<u32> {
        let old = self.play_cursor;
        self.play_cursor = cursor;
        let (moved, write) = cursor_moved(old, cursor, self.bytes, self.step)?;
        if let Some(at) = write {
            self.write_at = at;
        }
        Some(moved)
    }

    /// The write place moved on by `moved`, round the buffer.
    pub fn advance(&mut self, moved: u32) {
        self.write_at = round(moved.wrapping_add(self.write_at), self.bytes);
    }

    /// A part of `bytes` of which the file gave `read`: all of it, None; else the byte the
    /// rest is filled with (0, or 0x80 for eight bits), and a read that found nothing uses
    /// up `tail` (the player stops when it goes below 0).
    pub fn after_read(&mut self, bytes: u32, read: u32) -> Option<u8> {
        if read == bytes {
            return None;
        }
        if read == 0 {
            self.tail = self.tail.wrapping_sub(bytes as i32);
            if self.tail < 0 {
                self.playing = 0;
            }
        }
        Some(self.silence())
    }

    /// The byte of silence for the player's format.
    pub fn silence(&self) -> u8 {
        if self.format & 0x10 != 0 { 0x80 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream() {
        let mut s = AudioStream {
            bytes: 0x100,
            step: 0x40,
            play_cursor: 0xF0,
            write_at: 0x30,
            tail: 0x10,
            playing: 1,
            ..AudioStream::default()
        };
        assert_eq!(s.moved(0x10), Some(0x20));
        assert_eq!(s.moved(0x10), None);
        assert_eq!((s.moved(0x90), s.write_at), (Some(0x40), 0xD0));
        s.advance(0x40);
        assert_eq!(s.write_at, 0x10);
        assert_eq!(
            (s.after_read(8, 8), s.after_read(0x20, 0), s.playing),
            (None, Some(0), 0)
        );
    }

    #[derive(Default)]
    struct Test {
        playing: bool,
        volume: u32,
        name: u32,
        feeds_stop: u32,
        present: Vec<u32>,
        log: Vec<String>,
    }

    impl Player for Test {
        type Error = ();
        fn playing(&mut self) -> bool {
            self.playing
        }
        fn keep_volume(&mut self, v: u32) {
            self.volume = v;
        }
        fn start(&mut self) -> Result<bool, ()> {
            self.log.push(format!("start {}", self.name));
            self.playing = true;
            Ok(true)
        }
        fn feed(&mut self) -> Result<(), ()> {
            if self.feeds_stop > 0 {
                self.feeds_stop -= 1;
                self.playing = false;
            }
            Ok(())
        }
        fn stop(&mut self) -> Result<(), ()> {
            self.log.push("stop".into());
            self.playing = false;
            Ok(())
        }
        fn apply_volume(&mut self) -> Result<(), ()> {
            Ok(())
        }
    }

    impl Tunes for Test {
        fn name(&mut self, n: u32) {
            self.name = n;
        }
        fn exists(&mut self) -> Result<bool, ()> {
            Ok(self.present.contains(&self.name))
        }
        fn gave_up(&mut self, _: u32) {
            self.log.push("gave up".into());
        }
    }

    #[test]
    fn music_and_streams() {
        let mut p = Test::default();
        let mut m = Music::default();
        music_command(&mut p, &mut m, 4, 8, 0x30, 2).unwrap();
        assert_eq!(
            (p.log.as_slice(), m.next_tune, p.volume),
            (&["start 4".to_string()][..], 8, 0x30)
        );
        // The tune ends at its turn: the next one.
        p.feeds_stop = 1;
        music_command(&mut p, &mut m, 0, 0, 0, 1).unwrap();
        assert_eq!(p.log[1..], ["stop", "start 8"]);
        // Tune 0xA: turns 2 .. 5 then 0: 0xC is there.
        let mut p = Test {
            present: vec![0xC],
            ..Test::default()
        };
        let mut m = Music {
            turn: 2,
            ..Music::default()
        };
        music_command(&mut p, &mut m, 0xA, 0, 1, 2).unwrap();
        assert_eq!(
            (p.log.as_slice(), m.turn),
            (&["start 12".to_string()][..], 3)
        );
        // Every play stops at once: given up after seven.
        let mut p = Test {
            feeds_stop: 100,
            ..Test::default()
        };
        let mut m = Music::default();
        music_command(&mut p, &mut m, 4, 4, 1, 2).unwrap();
        assert_eq!(p.log.last().unwrap(), "stop");
        assert!(p.log.contains(&"gave up".to_string()));
        // The stream: no path, a stop.
        let mut p = Test::default();
        stream_command(&mut p, false, 5, 2).unwrap();
        assert_eq!(p.log, ["stop"]);
    }
}
