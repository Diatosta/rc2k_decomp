//! The championships' welcomes (converted 2026-10-21 from `menu::championship_welcome`
//! 0x47F314 and `menu::a8_welcome` 0x47F3C7): the picture, a key, and Escape asking about
//! leaving, a no starting over; the championship's with its speech.

use super::flow::{Call, Game, Picture, Var};

/// The picture that cannot be read: fatal error 0x80712345, no message.
pub const NO_PICTURE: (u32, &str) = (0x8071_2345, "");
/// Escape's key code.
pub const ESCAPE: u8 = 1;
/// How long the championship's picture waits for a key before the speech is watched (ms).
pub const FIRST_WAIT: u32 = 0xBB8;

/// The picture loaded and converted.
fn picture<G: Game>(e: &mut G, p: Picture) -> Result<(), G::Error> {
    if !e.load_picture(p)? {
        return Err(e.fatal(NO_PICTURE.0, NO_PICTURE.1));
    }
    e.run(Call::ConvertPicture)
}

/// `menu::championship_welcome`: nothing when a saved game goes on. Else the speech
/// started, the leave flags and the answer 0, the picture, a key dropped, one frame; then
/// three seconds for a key, and without one, keys taken while the speech plays and the
/// game is not quitting (its end or the quitting end it). A key but Escape stops the
/// speech and ends it; Escape stops it and asks about leaving: a yes ends it (the speech
/// stopped again), a no starts over.
pub fn championship_welcome<G: Game>(e: &mut G) -> Result<(), G::Error> {
    if e.get(Var::Continuing) == 1 {
        return Ok(());
    }
    loop {
        let channel = e.get(Var::SpeechChannel);
        e.run(Call::StartStream { channel })?;
        e.run(Call::WaitForCommands)?;
        e.set(Var::EventLeft, 0);
        e.set(Var::LeaveScreen, 0);
        e.set(Var::Answer, 0);
        picture(e, Picture::Championship)?;
        e.take_key()?;
        e.run(Call::RunFrame)?;
        let mut key = e.wait_or_key(FIRST_WAIT)?;
        if key.is_none() {
            loop {
                if !e.stream_playing()? || e.get(Var::Quitting) == 1 {
                    return Ok(());
                }
                key = e.take_key()?;
                if key.is_some() {
                    break;
                }
            }
        }
        if key != Some(ESCAPE) {
            break;
        }
        e.run(Call::StopStream)?;
        e.run(Call::ConfirmLeaving)?;
        if e.yes() {
            break;
        }
    }
    e.run(Call::StopStream)
}

/// `menu::a8_welcome`: the picture, frames of it until a key; Escape asks about leaving
/// (a no starts over); then one more key taken (dropped).
pub fn a8_welcome<G: Game>(e: &mut G) -> Result<(), G::Error> {
    loop {
        picture(e, Picture::A8)?;
        let key = loop {
            e.run(Call::RunFrame)?;
            if let Some(key) = e.take_key()? {
                break key;
            }
        };
        if key != ESCAPE {
            break;
        }
        e.run(Call::ConfirmLeaving)?;
        if e.yes() {
            break;
        }
    }
    e.take_key()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::flow::log::{Log, Said};
    use super::*;

    fn key(k: u8) -> Said {
        Said::Key(Some(k))
    }

    const NO_KEY: Said = Said::Key(None);

    #[test]
    fn championship() {
        let mut log = Log::default().with(Var::Continuing, 1);
        championship_welcome(&mut log).unwrap();
        assert!(log.steps.is_empty());

        // Escape at the wait, a no, then the speech's end.
        let mut log = Log::default()
            .reply("WaitOrKey", vec![key(ESCAPE)])
            .reply("StreamPlaying", vec![Said::Playing(false)]);
        championship_welcome(&mut log).unwrap();
        assert_eq!(log.count("ConfirmLeaving"), 1);
        assert_eq!(log.count("StopStream"), 1);
        assert_eq!(log.count("StartStream { channel: 0 }"), 2);

        // A key while the speech plays.
        let mut log = Log::default().reply("TakeKey", vec![key(0), NO_KEY, key(0x1C)]);
        championship_welcome(&mut log).unwrap();
        assert_eq!(log.steps.last().unwrap(), "StopStream");

        let mut log = Log::default().reply("LoadPicture", vec![Said::Unreadable]);
        assert!(championship_welcome(&mut log).is_err());
    }

    #[test]
    fn a8() {
        let mut log = Log::default()
            .reply("TakeKey", vec![NO_KEY, key(ESCAPE), key(5), key(0)])
            .on("ConfirmLeaving", vec![vec![(Var::Answer, 0)]]);
        a8_welcome(&mut log).unwrap();
        assert_eq!(log.count("RunFrame"), 3);
        assert_eq!(log.count("TakeKey"), 4);
        assert_eq!(log.count("LoadPicture(A8)"), 2);
    }
}
