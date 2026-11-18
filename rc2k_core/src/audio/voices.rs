//! The audio thread's sounds: sample records, voices and mixer channels as the game lays them out,
//! reached through a `Pool` by handle (the game's addresses), and the playing and stopping lists
//! linked by `Voice::next`.
//!
//! A sound number picks a sample record (`SAMPLES`); a loaded record points at its first voice
//! (a pool of 0x200); a sound that can sound more than once at a time has more voices,
//! linked by `next_copy`. A voice that sounds is on the playing list and holds a mixer
//! channel; a voice put out of use goes to the stopping list for the next turn to stop.
//! What a command changes in a voice is marked in its `dirty` bits for the turn.

use crate::Invalid;
use crate::audio::commands::{Setting, Start};

pub const BASE_SAMPLES: u32 = 0x1CD;
/// The engine sounds' records (numbers below this) that the third and fourth players' cars
/// have copies of, after the base records: the copy of sound `n` is `BASE_SAMPLES + n`.
pub const PLAYER_COPIES: u32 = 0xD0;
/// The sample records: the base ones, then the copies for the third and fourth cars.
pub const SAMPLES: u32 = BASE_SAMPLES + PLAYER_COPIES;
pub const VOICES: u32 = 0x200;
/// The mixer channels that are cleared (`audio::clear_channels`).
pub const CHANNELS_CLEARED: u32 = 0x10;

/// "As the sample record says" in a command.
pub const DEFAULT: u32 = 0xFFFF_FF91;
/// "The voice's own frequency".
pub const OWN_FREQUENCY: u32 = 0xFFFF_FF90;
/// "As the sample record says", for a place's float.
pub const DEFAULT_PLACE: u32 = 0xFF80_0000;
/// The sound mode's bit: places in 3D.
pub const MODE_3D: u32 = 0x10_0000;

pub const DIRTY_VOLUME: u32 = 1;
pub const DIRTY_PAN: u32 = 2;
pub const DIRTY_FREQUENCY: u32 = 4;
pub const DIRTY_PLACE: u32 = 8;
pub const DIRTY_PLAY: u32 = 0x10;
pub const DIRTY_RESTART: u32 = 0x20;
/// `Voice::flags`: the voice's sample is loaded.
pub const LOADED: u32 = 0x10;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampleRecord {
    pub number: u32,
    pub group: u32,
    /// The first voice (0: not loaded).
    pub first_voice: u32,
    pub data_key: u32,
    pub priority: u32,
    pub voice_kind: u32,
    /// How many voices to make for it.
    pub copies: i32,
    /// The sample's volume scale out of 0x40 (`audio::prepare_sample`).
    pub scale: u32,
    /// Names a volume factor, read through the caller's `factor` (0: 0x40).
    pub volume_factor: u32,
    /// Its voices' 3D minimum distance (an f32).
    pub min_distance: u32,
    pub volume: u32,
    pub pan: u32,
    pub frequency: u32,
    pub place: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voice {
    /// 1 a buffer of its own, 2 the software mixer's; `LOADED`.
    pub flags: u32,
    /// Its sample record's number.
    pub sound: u32,
    pub buffer: u32,
    /// The voice's DirectSound 3D buffer interface; 0 without one (`audio::buffer_settings`,
    /// `set_buffer_min_distance`).
    pub buffer_3d: u32,
    pub data_key: u32,
    /// The voice whose sample data this one shares.
    pub source: u32,
    pub next_copy: u32,
    pub data: u32,
    pub length: u32,
    pub own_frequency: u32,
    /// Low four bits: the sample's shift.
    pub format: u32,
    /// The sample's frames.
    pub frames: u32,
    /// The record's `scale`.
    pub scale: u32,
    pub priority: i32,
    /// 0 once, 1 looping, 3 restarted each time.
    pub kind: u32,
    /// 0 free, 1 started, 2 sounding.
    pub state: u32,
    pub dirty: u32,
    /// Its mixer channel (0: none).
    pub channel: u32,
    /// The playing or stopping list's link.
    pub next: u32,
    pub volume_factor: u32,
    /// The 3D buffer's minimum distance (an f32).
    pub min_distance: u32,
    pub volume: u32,
    pub pan: i32,
    pub pan_sign: i32,
    pub frequency: u32,
    pub place: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MixerChannel {
    pub used: u32,
    pub gap_4: [u32; 2],
    pub voice: u32,
    pub format: u32,
    pub data: u32,
    pub length: u32,
    pub kind: u32,
    pub gap_20: [u32; 3],
    pub position: u32,
    pub position_fraction: u32,
    pub step: u32,
    pub step_fraction: u32,
    pub left: i32,
    pub right: i32,
    /// +0x44 .. +0x58: five dwords nothing reads (cleared when the channel is released).
    pub unused_44: [u32; 5],
}

const _: () = assert!(std::mem::size_of::<SampleRecord>() == 0x40);
const _: () = assert!(std::mem::size_of::<Voice>() == 0x70);
const _: () = assert!(std::mem::size_of::<MixerChannel>() == 0x58);

/// Where voices, channels and records are, by handle: read as copies, a change written back
/// at once (`update_voice`, `update_channel`), so nothing holds a record between changes.
pub trait Pool {
    /// Voice `at` as it is now.
    fn voice(&self, at: u32) -> Voice;
    /// Voice `at` replaced by `v`.
    fn set_voice(&mut self, at: u32, v: Voice);
    /// Channel `at` as it is now.
    fn channel(&self, at: u32) -> MixerChannel;
    /// Channel `at` replaced by `c`.
    fn set_channel(&mut self, at: u32, c: MixerChannel);
    /// Channel `n`'s handle.
    fn channel_at(&self, n: u32) -> u32;
    /// Sample record `n` as it is now.
    fn record(&self, n: u32) -> SampleRecord;

    /// Voice `at` read, changed by `change` and written back; `change`'s answer.
    fn update_voice<R>(&mut self, at: u32, change: impl FnOnce(&mut Voice) -> R) -> R {
        let mut v = self.voice(at);
        let answer = change(&mut v);
        self.set_voice(at, v);
        answer
    }

    /// Channel `at` read, changed by `change` and written back.
    fn update_channel(&mut self, at: u32, change: impl FnOnce(&mut MixerChannel)) {
        let mut c = self.channel(at);
        change(&mut c);
        self.set_channel(at, c);
    }
}

/// The lists' heads and the channels' counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lists {
    pub playing: u32,
    pub stopping: u32,
    pub channels_used: u32,
    pub channel_count: u32,
}

/// A place that points at a voice: a list's head, or a voice's `next`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Playing,
    Stopping,
    After(u32),
}

fn get<P: Pool>(pool: &mut P, lists: &Lists, link: Link) -> u32 {
    match link {
        Link::Playing => lists.playing,
        Link::Stopping => lists.stopping,
        Link::After(v) => pool.voice(v).next,
    }
}

fn set<P: Pool>(pool: &mut P, lists: &mut Lists, link: Link, value: u32) {
    match link {
        Link::Playing => lists.playing = value,
        Link::Stopping => lists.stopping = value,
        Link::After(v) => pool.update_voice(v, |v| v.next = value),
    }
}

/// The voices of a list from `head`. Refused: no end within 0x200.
pub fn list<P: Pool>(pool: &mut P, head: u32) -> Result<Vec<u32>, Invalid> {
    let mut out = Vec::new();
    let mut at = head;
    while at != 0 {
        out.push(at);
        if out.len() > VOICES as usize {
            return Err(Invalid("a list of voices without its end"));
        }
        at = pool.voice(at).next;
    }
    Ok(out)
}

/// The voice at `link` taken off its list to the stopping list's front, its channel freed.
fn to_stopping<P: Pool>(pool: &mut P, lists: &mut Lists, link: Link, at: u32) {
    let channel = pool.voice(at).channel;
    pool.update_channel(channel, |c| c.used = 0);
    pool.update_voice(at, |v| v.channel = 0);
    let next = pool.voice(at).next;
    set(pool, lists, link, next);
    pool.update_voice(at, |v| v.next = lists.stopping);
    lists.stopping = at;
}

/// Command 7: the playing list put in front of the stopping list and emptied, no channel
/// counted (the channels keep their marks: the turn frees them). Refused: a playing list
/// without its end.
pub fn stop_all<P: Pool>(pool: &mut P, lists: &mut Lists) -> Result<(), Invalid> {
    let playing = list(pool, lists.playing)?;
    let Some(&last) = playing.last() else {
        return Ok(());
    };
    pool.update_voice(last, |v| v.next = lists.stopping);
    lists.stopping = lists.playing;
    lists.playing = 0;
    lists.channels_used = 0;
    Ok(())
}

/// The first 0x10 channels made free: `used`, `voice` and the five dwords from +0x44 0.
pub fn clear_channels<P: Pool>(pool: &mut P) {
    for n in 0..CHANNELS_CLEARED {
        let at = pool.channel_at(n);
        pool.update_channel(at, |c| {
            c.used = 0;
            c.voice = 0;
            c.unused_44 = [0; 5];
        });
    }
}

/// The first voice (of `count`, handles from `voice_at`) whose `data_key` is `key`.
pub fn find_shared<P: Pool>(
    pool: &mut P,
    count: u32,
    voice_at: impl Fn(u32) -> u32,
    key: u32,
) -> Option<u32> {
    (0..count)
        .map(voice_at)
        .find(|&at| pool.voice(at).data_key == key)
}

/// The fatal error when the pool is used up.
pub const POOL_USED_UP: u32 = 0x80A2_8501;

/// The next voice of the pool: its place, the count one more. Used up: `None` (a fatal error).
pub fn new_voice(count: &mut u32) -> Option<u32> {
    if *count >= VOICES {
        return None;
    }
    *count += 1;
    Some(*count - 1)
}

/// Where a voice and a command lead.
enum Plan {
    Nothing,
    FreeChannel { voice: u32, channel: u32 },
    Steal { voice: u32, victim: Link },
    Stop { voice: u32, link: Link },
    Change { voice: u32 },
}

fn find_playing<P: Pool>(pool: &mut P, lists: &Lists, voice: u32) -> Result<Option<Link>, Invalid> {
    let mut link = Link::Playing;
    for _ in 0..=VOICES {
        let at = get(pool, lists, link);
        if at == 0 {
            return Ok(None);
        }
        if at == voice {
            return Ok(Some(link));
        }
        link = Link::After(at);
    }
    Err(Invalid("a playing list without its end"))
}

fn plan_start<P: Pool>(pool: &mut P, lists: &Lists, mut voice: u32) -> Result<Plan, Invalid> {
    while pool.voice(voice).state != 0 {
        voice = pool.voice(voice).next_copy;
        if voice == 0 {
            return Ok(Plan::Nothing);
        }
    }
    if lists.channels_used < lists.channel_count {
        let free = (0..lists.channel_count).find_map(|n| {
            let c = pool.channel_at(n);
            (pool.channel(c).used == 0).then_some(c)
        });
        return Ok(free.map_or(Plan::Nothing, |channel| Plan::FreeChannel {
            voice,
            channel,
        }));
    }
    // The playing voice of the lowest priority, if below this one's.
    let mine = pool.voice(voice).priority;
    let mut lowest = mine;
    let mut victim = None;
    let mut link = Link::Playing;
    let mut at = lists.playing;
    if at == 0 {
        return Err(Invalid("no channel left and no voice playing"));
    }
    for _ in 0..=VOICES {
        if lowest > pool.voice(at).priority {
            lowest = pool.voice(at).priority;
            victim = Some(link);
        }
        link = Link::After(at);
        at = get(pool, lists, link);
        if at == 0 {
            return Ok(match victim {
                Some(victim) if lowest != mine => {
                    let v = get(pool, lists, victim);
                    if pool.voice(v).channel == 0 {
                        Plan::Nothing
                    } else {
                        Plan::Steal { voice, victim }
                    }
                }
                _ => Plan::Nothing,
            });
        }
    }
    Err(Invalid("a playing list without its end"))
}

fn plan<P: Pool>(pool: &mut P, lists: &Lists, sound: i32, cmd: &Start) -> Result<Plan, Invalid> {
    if sound < 0 {
        return Ok(Plan::Nothing);
    }
    if sound as u32 >= SAMPLES {
        return Err(Invalid("a sound past the sample records"));
    }
    let first = pool.record(sound as u32).first_voice;
    if first == 0 || pool.voice(first).flags & LOADED == 0 {
        return Ok(Plan::Nothing);
    }
    let silent = cmd.volume == Setting::Value(0);
    if !silent && pool.voice(first).kind == 0 {
        return plan_start(pool, lists, first);
    }
    match find_playing(pool, lists, first)? {
        None if silent || cmd.only_if_playing => Ok(Plan::Nothing),
        None => plan_start(pool, lists, first),
        Some(link) if silent => Ok(Plan::Stop { voice: first, link }),
        Some(_) => Ok(Plan::Change { voice: first }),
    }
}

/// The command's settings into the voice, each change marked dirty; kind 3 a restart; the
/// play bit.
fn change<P: Pool>(pool: &mut P, at: u32, cmd: &Start, mode: u32) {
    let sound = pool.voice(at).sound;
    let record = pool.record(sound);
    pool.update_voice(at, |v| change_voice(v, &record, cmd, mode));
}

fn change_voice(v: &mut Voice, record: &SampleRecord, cmd: &Start, mode: u32) {
    let mut dirty = v.dirty;
    let volume = cmd.volume.resolve(record.volume).min(0x40);
    if volume != v.volume {
        dirty |= DIRTY_VOLUME;
        v.volume = volume;
    }
    let pan = cmd.pan.resolve(record.pan);
    let (low, high) = (pan as u8, (pan >> 8) as u8);
    v.pan_sign = if (low ^ high) & 0x80 != 0 { -1 } else { 0 };
    let pan = (low as i8 as i32).clamp(-0x40, 0x40);
    if pan != v.pan {
        dirty |= DIRTY_PAN;
        v.pan = pan;
    }
    let mut frequency = cmd.frequency.resolve(record.frequency);
    if frequency == OWN_FREQUENCY {
        frequency = v.own_frequency;
    }
    let frequency = (frequency as i32).clamp(100, 200_000) as u32;
    if frequency != v.frequency {
        dirty |= DIRTY_FREQUENCY;
        v.frequency = frequency;
    }
    if mode & MODE_3D != 0 {
        for i in 0..3 {
            let p = match cmd.place {
                Some(place) if place[i] != DEFAULT_PLACE => place[i],
                _ => record.place[i],
            };
            if p != v.place[i] {
                dirty |= DIRTY_PLACE;
                v.place[i] = p;
            }
        }
    }
    if v.kind == 3 {
        dirty |= DIRTY_RESTART;
    }
    v.dirty = dirty | DIRTY_PLAY;
}

fn start<P: Pool>(pool: &mut P, lists: &mut Lists, at: u32) {
    pool.update_voice(at, |v| v.next = lists.playing);
    lists.playing = at;
    pool.update_voice(at, |v| v.state = 1);
    let channel = pool.voice(at).channel;
    pool.update_channel(channel, |c| c.voice = at);
}

/// Command 1: a sound started, changed or stopped. With a volume and a voice of kind 0
/// (or, another kind or no volume, a voice not playing, unless the command is only_if_playing)
/// 31) its first free copy gets a channel: a free one while fewer are used than there are
/// (counted), else the channel of the playing voice of the lowest priority below its own
/// (none: nothing), which goes to the stopping list; the voice to the playing list's
/// front, started. A playing voice with no volume is stopped (its channel freed and
/// counted down). Then for a voice started or playing the settings (`change`). Refused,
/// before anything: a number of 0x1CD or more (its own or the voice's); no channels left
/// and none playing; a playing list without its end; a voice to stop without its channel.
pub fn start_voice<P: Pool>(
    pool: &mut P,
    lists: &mut Lists,
    mode: u32,
    sound: i32,
    cmd: &Start,
) -> Result<(), Invalid> {
    let plan = plan(pool, lists, sound, cmd)?;
    let target = match plan {
        Plan::Nothing | Plan::Stop { .. } => None,
        Plan::FreeChannel { voice, .. } | Plan::Steal { voice, .. } | Plan::Change { voice } => {
            Some(voice)
        }
    };
    if let Some(at) = target
        && pool.voice(at).sound >= SAMPLES
    {
        return Err(Invalid("a voice whose sound is past the sample records"));
    }
    if let Plan::Stop { voice, .. } = plan
        && pool.voice(voice).channel == 0
    {
        return Err(Invalid("a playing voice without its channel"));
    }
    match plan {
        Plan::Nothing => {}
        Plan::Stop { voice, link } => {
            to_stopping(pool, lists, link, voice);
            lists.channels_used = lists.channels_used.wrapping_sub(1);
        }
        Plan::FreeChannel { voice, channel } => {
            pool.update_channel(channel, |c| c.used = 1);
            pool.update_voice(voice, |v| v.channel = channel);
            lists.channels_used = lists.channels_used.wrapping_add(1);
            start(pool, lists, voice);
            change(pool, voice, cmd, mode);
        }
        Plan::Steal { voice, victim } => {
            let v = get(pool, lists, victim);
            let channel = pool.voice(v).channel;
            pool.update_voice(voice, |x| x.channel = channel);
            pool.update_voice(v, |x| x.channel = 0);
            let next = pool.voice(v).next;
            set(pool, lists, victim, next);
            pool.update_voice(v, |x| x.next = lists.stopping);
            lists.stopping = v;
            start(pool, lists, voice);
            change(pool, voice, cmd, mode);
        }
        Plan::Change { voice } => change(pool, voice, cmd, mode),
    }
    Ok(())
}

/// What the turn's update asks of a voice's buffer, or of the 3D sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Whether the voice's buffer still plays (the answer).
    IsPlaying(u32),
    Frequency(u32),
    Volume(u32),
    Pan(u32),
    Place(u32),
    Rewind(u32),
    Play(u32),
    Stop(u32),
    /// The EAX room set.
    Room,
    /// The deferred 3D settings carried out.
    Commit,
}

/// The EAX room's variables and the 3D commit flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Room {
    pub asked: u32,
    pub set: u32,
    pub commit: u32,
}

/// The sound mode's bit: with EAX.
pub const MODE_EAX: u32 = 0x100_0000;

fn stopping_pass<P: Pool, E>(
    pool: &mut P,
    lists: &mut Lists,
    step: &mut impl FnMut(Step) -> Result<bool, E>,
) -> Result<(), E> {
    let mut link = Link::Stopping;
    loop {
        let at = get(pool, lists, link);
        if at == 0 {
            return Ok(());
        }
        if pool.voice(at).state == 2 {
            step(Step::Stop(at))?;
        }
        pool.update_voice(at, |v| v.state = 0);
        let channel = pool.voice(at).channel;
        if channel != 0 {
            pool.update_channel(channel, |c| c.used = 0);
            pool.update_voice(at, |v| v.channel = 0);
        }
        set(pool, lists, link, 0);
        link = Link::After(at);
    }
}

fn apply<P: Pool, E>(
    pool: &mut P,
    at: u32,
    mode: u32,
    room: &mut Room,
    step: &mut impl FnMut(Step) -> Result<bool, E>,
) -> Result<(), E> {
    let x = pool.voice(at);
    if x.buffer != 0 {
        if x.dirty & DIRTY_FREQUENCY != 0 {
            step(Step::Frequency(at))?;
        }
        if x.dirty & DIRTY_VOLUME != 0 {
            step(Step::Volume(at))?;
        }
        if x.dirty & DIRTY_PAN != 0 {
            step(Step::Pan(at))?;
        }
        if mode & MODE_3D != 0 && x.buffer_3d != 0 && x.dirty & DIRTY_PLACE != 0 {
            step(Step::Place(at))?;
            room.commit = 1;
        }
        if x.dirty & DIRTY_RESTART != 0 || x.state != 2 {
            step(Step::Rewind(at))?;
            if pool.voice(at).state != 2 {
                step(Step::Play(at))?;
                pool.update_voice(at, |v| v.state = 2);
            }
        }
    }
    pool.update_voice(at, |v| v.dirty = 0);
    Ok(())
}

/// The turn's update with voices of their own buffers: the stopping list stopped and
/// emptied (each voice free, its channel freed); each playing voice's marked settings
/// carried to its buffer, one not sounding (or to restart) rewound and played; one that
/// does not loop and has ended goes to the stopping list (a channel fewer); the stopping
/// list again; then in 3D the EAX room when asked, and the
/// deferred settings committed. Refused (before anything, as `Invalid` through `E`): a
/// list without its end; a playing voice that does not loop without its channel or
/// buffer; a marked voice's volume or pan past the volume table; a voice to stop without
/// its buffer.
pub fn update_buffers<P: Pool, E: From<Invalid>>(
    pool: &mut P,
    lists: &mut Lists,
    mode: u32,
    room: &mut Room,
    mut step: impl FnMut(Step) -> Result<bool, E>,
) -> Result<(), E> {
    let playing = list(pool, lists.playing)?;
    let stopping = list(pool, lists.stopping)?;
    for &at in &playing {
        let x = pool.voice(at);
        if x.kind != 1 && (x.channel == 0 || x.buffer == 0) {
            return Err(Invalid("a playing voice without its channel or buffer").into());
        }
        let level = (x.volume as i32).wrapping_mul(x.volume_factor as i32) >> 6;
        if x.buffer != 0
            && x.dirty != 0
            && (!(0..=0x40).contains(&level) || !(-0x40..=0x40).contains(&x.pan))
        {
            return Err(Invalid("a volume or pan past the volume table").into());
        }
    }
    if stopping.iter().any(|&at| {
        let v = pool.voice(at);
        v.state == 2 && v.buffer == 0
    }) {
        return Err(Invalid("a voice to stop without its buffer").into());
    }
    stopping_pass(pool, lists, &mut step)?;
    let mut link = Link::Playing;
    loop {
        let at = get(pool, lists, link);
        if at == 0 {
            break;
        }
        if pool.voice(at).dirty != 0 {
            apply(pool, at, mode, room, &mut step)?;
        }
        if pool.voice(at).kind != 1 && !step(Step::IsPlaying(at))? {
            to_stopping(pool, lists, link, at);
            lists.channels_used = lists.channels_used.wrapping_sub(1);
            continue;
        }
        link = Link::After(at);
    }
    stopping_pass(pool, lists, &mut step)?;
    if mode & MODE_3D != 0 {
        if mode & MODE_EAX != 0 && room.asked != room.set {
            room.set = room.asked;
            step(Step::Room)?;
        }
        if room.commit == 1 {
            step(Step::Commit)?;
            room.commit = 0;
        }
    }
    Ok(())
}

/// What a voice takes from the sample file read for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileSample {
    pub data: u32,
    pub length: u32,
    pub format: u32,
    pub frames: u32,
    pub rate: u32,
}

/// `audio::fill_voice`: voice `at` made a voice of `record`: its sample from the file, or
/// with `shared` from that voice (then added at the end of the record's chain unless it is
/// on it); the rest from the record, flags 1 with DirectSound's buffers (`own_buffers`)
/// else 2, the volume factor 0x40 or what the record's points at (`factor`, asked only
/// then). Refused, before anything is written: a shared voice for a record without voices,
/// a chain without its end.
pub fn fill_voice<P: Pool>(
    pool: &mut P,
    at: u32,
    record: &SampleRecord,
    shared: u32,
    file: &FileSample,
    own_buffers: bool,
    factor: impl Fn(u32) -> u32,
) -> Result<(), Invalid> {
    if shared != 0 {
        // The record's chain, to find its end, before anything is written.
        if record.first_voice == 0 {
            return Err(Invalid("a shared voice for a record without voices"));
        }
        let mut last = record.first_voice;
        let mut steps = 0;
        while last != at && pool.voice(last).next_copy != 0 {
            last = pool.voice(last).next_copy;
            steps += 1;
            if steps > VOICES {
                return Err(Invalid("a record's voices without their end"));
            }
        }
        let s = pool.voice(shared);
        pool.update_voice(at, |v| {
            v.data = s.data;
            v.length = s.length;
            v.format = s.format;
            v.frames = s.frames;
            v.own_frequency = s.own_frequency;
            v.source = shared;
        });
        if last != at {
            pool.update_voice(last, |v| v.next_copy = at);
        }
        pool.update_voice(at, |v| {
            v.next_copy = 0;
            v.data_key = 0;
        });
    } else {
        pool.update_voice(at, |v| {
            v.data = file.data;
            v.length = file.length;
            v.format = file.format;
            v.frames = file.frames;
            v.own_frequency = file.rate;
            v.source = 0;
            v.next_copy = 0;
            v.data_key = record.data_key;
        });
    }
    // The record's level, asked only when it has one (it reads no voice).
    let volume_factor = if record.volume_factor != 0 {
        factor(record.volume_factor)
    } else {
        0x40
    };
    pool.update_voice(at, |v| {
        set_from_record(v, record, own_buffers, volume_factor);
    });
    Ok(())
}

/// What `fill_voice` takes from the record, the voice made free.
fn set_from_record(v: &mut Voice, record: &SampleRecord, own_buffers: bool, volume_factor: u32) {
    v.flags = if own_buffers { 1 } else { 2 };
    v.sound = record.number;
    v.kind = record.voice_kind;
    v.scale = record.scale;
    v.priority = record.priority as i32;
    v.volume = record.volume;
    v.pan = record.pan as u8 as i8 as i32;
    v.pan_sign = 0;
    v.frequency = if record.frequency == OWN_FREQUENCY {
        v.own_frequency
    } else {
        record.frequency
    };
    v.place = record.place;
    v.min_distance = record.min_distance;
    v.volume_factor = volume_factor;
    v.state = 0;
    v.dirty |= 0xF;
    v.channel = 0;
    v.next = 0;
}

/// The sound mode's bits a voice's own buffer is made by: 3D, 3D in hardware.
pub const MODE_HARDWARE_3D: u32 = 0x80_0000;

/// `audio::make_buffer`'s buffer description flags for a voice: the frequency, pan and
/// volume controls (0xE0), in software (8) unless 3D in hardware, static (2) for the
/// voice's flags bit 0, 3D (0x10) in 3D.
pub fn buffer_flags(mode: u32, voice_flags: u32) -> u32 {
    let mut flags = 0xE0;
    if !(mode & MODE_3D != 0 && mode & MODE_HARDWARE_3D != 0) {
        flags |= 8;
    }
    if voice_flags & 1 != 0 {
        flags |= 2;
    }
    if mode & MODE_3D != 0 {
        flags |= 0x10;
    }
    flags
}

/// `audio::release_voice`'s end: a voice whose buffers are given back made empty. Its
/// sound, buffers, volume, pan, frequency and place stay as they are; its own frequency is
/// 44100, its scale and volume factor 0x40, its minimum distance 1.0.
pub fn empty(v: &mut Voice) {
    v.flags = 0;
    v.data_key = 0;
    v.source = 0;
    v.next_copy = 0;
    v.data = 0;
    v.length = 0;
    v.own_frequency = 44100;
    v.format = 0;
    v.frames = 0;
    v.scale = 0x40;
    v.priority = 0;
    v.kind = 0;
    v.state = 0;
    v.dirty = 0;
    v.channel = 0;
    v.next = 0;
    v.volume_factor = 0x40;
    v.min_distance = 1.0f32.to_bits();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Handles: voices 0x1000 + 0x70 n, channels 0x8000 + 0x58 n.
    struct Test {
        voices: Vec<Voice>,
        channels: Vec<MixerChannel>,
        records: Vec<SampleRecord>,
    }

    impl Test {
        fn voice_mut(&mut self, at: u32) -> &mut Voice {
            &mut self.voices[((at - 0x1000) / 0x70) as usize]
        }
        fn channel_mut(&mut self, at: u32) -> &mut MixerChannel {
            &mut self.channels[((at - 0x8000) / 0x58) as usize]
        }
    }

    impl Pool for Test {
        fn voice(&self, at: u32) -> Voice {
            self.voices[((at - 0x1000) / 0x70) as usize]
        }
        fn set_voice(&mut self, at: u32, v: Voice) {
            *self.voice_mut(at) = v;
        }
        fn channel(&self, at: u32) -> MixerChannel {
            self.channels[((at - 0x8000) / 0x58) as usize]
        }
        fn set_channel(&mut self, at: u32, c: MixerChannel) {
            *self.channel_mut(at) = c;
        }
        fn channel_at(&self, n: u32) -> u32 {
            0x8000 + 0x58 * n
        }
        fn record(&self, n: u32) -> SampleRecord {
            self.records[n as usize]
        }
    }

    fn v(n: u32) -> u32 {
        0x1000 + 0x70 * n
    }

    #[test]
    fn voices_start_steal_stop() {
        let mut t = Test {
            voices: vec![Voice::default(); 8],
            channels: vec![MixerChannel::default(); 0x10],
            records: vec![SampleRecord::default(); 4],
        };
        for n in 0..4u32 {
            t.records[n as usize].first_voice = v(n);
            t.records[n as usize].volume = 0x20;
            *t.voice_mut(v(n)) = Voice {
                flags: LOADED,
                sound: n,
                priority: n as i32,
                ..Voice::default()
            };
        }
        let mut lists = Lists {
            channel_count: 1,
            ..Lists::default()
        };
        let cmd = |volume| Start::from_words(0, volume, DEFAULT, DEFAULT, [DEFAULT, 0, 0]);
        start_voice(&mut t, &mut lists, 0, 1, &cmd(DEFAULT)).unwrap();
        assert_eq!((lists.playing, lists.channels_used), (v(1), 1));
        assert_eq!(t.voice(v(1)).volume, 0x20);
        assert_eq!(t.voice(v(1)).dirty & DIRTY_PLAY, DIRTY_PLAY);
        // No channel left: 3 takes 1's (a lower priority).
        start_voice(&mut t, &mut lists, 0, 3, &cmd(5)).unwrap();
        assert_eq!((lists.playing, lists.stopping), (v(3), v(1)));
        assert_eq!(t.voice(v(3)).channel, 0x8000);
        // 0 has the lowest priority: nothing.
        start_voice(&mut t, &mut lists, 0, 0, &cmd(5)).unwrap();
        assert_eq!(lists.playing, v(3));
        // Stopped with no volume.
        start_voice(&mut t, &mut lists, 0, 3, &cmd(0)).unwrap();
        assert_eq!(
            (lists.playing, lists.stopping, lists.channels_used),
            (0, v(3), 0)
        );
        assert!(start_voice(&mut t, &mut lists, 0, SAMPLES as i32, &cmd(5)).is_err());
        // The turn: the stopping list emptied.
        t.voice_mut(v(3)).state = 2;
        t.voice_mut(v(3)).buffer = 9;
        let mut steps = Vec::new();
        update_buffers::<_, Invalid>(&mut t, &mut lists, 0, &mut Room::default(), |s| {
            steps.push(s);
            Ok(true)
        })
        .unwrap();
        assert_eq!(steps, [Step::Stop(v(3))]);
        assert_eq!((lists.stopping, t.voice(v(3)).state), (0, 0));
        // stop_all.
        lists.playing = v(0);
        t.voice_mut(v(0)).next = v(2);
        lists.stopping = v(1);
        stop_all(&mut t, &mut lists).unwrap();
        assert_eq!(
            (lists.playing, lists.stopping, t.voice(v(2)).next),
            (0, v(0), v(1))
        );
        let mut count = 0x1FF;
        assert_eq!(new_voice(&mut count), Some(0x1FF));
        assert_eq!(new_voice(&mut count), None);
    }

    #[test]
    fn filled_and_emptied() {
        let mut t = Test {
            voices: vec![Voice::default(); 4],
            channels: vec![],
            records: vec![],
        };
        let record = SampleRecord {
            number: 7,
            first_voice: v(0),
            data_key: 0x55,
            priority: 3,
            voice_kind: 1,
            scale: 0x30,
            volume_factor: 0x900,
            volume: 0x20,
            pan: 0xF0,
            frequency: OWN_FREQUENCY,
            place: [1, 2, 3],
            ..SampleRecord::default()
        };
        let file = FileSample {
            data: 0x100,
            length: 0x200,
            format: 0x121,
            frames: 0x80,
            rate: 22050,
        };
        fill_voice(&mut t, v(0), &record, 0, &file, true, |_| 0x22).unwrap();
        let first = t.voice(v(0));
        assert_eq!(
            (
                first.data,
                first.data_key,
                first.flags,
                first.pan,
                first.frequency
            ),
            (0x100, 0x55, 1, -0x10, 22050)
        );
        assert_eq!(
            (first.volume_factor, first.dirty, first.place),
            (0x22, 0xF, [1, 2, 3])
        );
        fill_voice(&mut t, v(1), &record, v(0), &file, false, |_| 0x22).unwrap();
        assert_eq!(t.voice(v(0)).next_copy, v(1));
        let second = t.voice(v(1));
        assert_eq!((second.source, second.data_key, second.flags), (v(0), 0, 2));
        assert!(
            fill_voice(
                &mut t,
                v(2),
                &SampleRecord::default(),
                v(0),
                &file,
                true,
                |_| 0
            )
            .is_err()
        );
        assert_eq!(buffer_flags(0, 1), 0xEA);
        assert_eq!(buffer_flags(MODE_3D, 0), 0xF8);
        assert_eq!(buffer_flags(MODE_3D | MODE_HARDWARE_3D, 1), 0xF2);
        let mut e = second;
        empty(&mut e);
        assert_eq!(
            (e.sound, e.scale, e.own_frequency, e.min_distance),
            (7, 0x40, 44100, 0x3F80_0000)
        );
    }
}
