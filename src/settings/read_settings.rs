//! `settings::read_settings` (0x4114A6): reads `var\ral.cfg` into the game's settings.
//!
//! In:  nothing
//! Out: every register preserved
//! `read_text` on `var\ral.cfg`; if it reads, `sub_4114FF` (0x4114FF, one caller, here as
//! `apply_settings`): for each of the 77 entries of the table at 0x518C09 (a key, then the
//! address of its variable), if `find_key` finds the key, the variable becomes
//! `parse_number` of its value. A key that is not in the file leaves its variable alone;
//! a key without a number makes it 0. Two keys share a variable twice (`noaudio` and
//! `nosound`, `allsecretcars` and `secretcar`): the later one in the table wins.
//! Callers: `start` and `sub_411239`.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;

// ---- game data ----

/// "var\ral.cfg" (0x5295CC in the image).
static PATH: &[u8; 12] = b"var\\ral.cfg\0";
/// aNotaskswitch (0x518C09 .. 0x5190F1 in the image): the settings, each a key and the
/// IDA address of its dword (`crate::moved::place` answers where it lives now); the table's
/// bytes are these entries, key, NUL, address (`tests::table_is_the_games`).
const ENTRIES: [(&[u8], u32); 77] = [
    (b"notaskswitch", 0x518384),
    (b"noprichange", 0x518388),
    (b"ignoremmx", 0x5183AC),
    (b"ignorekatmai", 0x5183B4),
    (b"ignoreamdk3d", 0x5183BC),
    (b"altmemenable", 0x4E0584),
    (b"altmemheapsize", 0x4E0588),
    (b"noaudio", 0x4F82B4),
    (b"nosound", 0x4F82B4),
    (b"nocdaudio", 0x518104),
    (b"nomusic", 0x5069BC),
    (b"audiobasefreq", 0x4F82D4),
    (b"ignoreds3d", 0x506B42),
    (b"ignoredsv", 0x506C50),
    (b"ignoreeax", 0x506C6C),
    (b"ignorea3d", 0x506C5C),
    (b"wavheapsize", 0x5068EC),
    (b"bundleheapsize", 0x51B980),
    (b"imageheapsize", 0x519554),
    (b"gamegfxheapsize", 0x5BE894),
    (b"forcedx5", 0x5AB958),
    (b"forceno3dcard", 0x5AB94C),
    (b"ignoreprimary", 0x5AB950),
    (b"ignoresecondary", 0x5AB954),
    (b"agpmode", 0x5AB95C),
    (b"force3dtrans", 0x5AB97C),
    (b"force3dalpha", 0x5AB984),
    (b"force3dshadow", 0x5AB980),
    (b"force3dfogging", 0x5AB988),
    (b"textflashmode", 0x4F3F74),
    (b"videomodemain", 0x4F4944),
    (b"videomodegame", 0x4F4948),
    (b"v3driverfix", 0x624238),
    (b"nopngdecode", 0x51B97C),
    (b"nojoy", 0x4F68DC),
    (b"joymode", 0x4F68E0),
    (b"joycode", 0x4F68E4),
    (b"joylowerdz", 0x4F68E8),
    (b"joyupperdz", 0x4F68EC),
    (b"noforcefeedback", 0x4F68F4),
    (b"joyffgain", 0x4F68F8),
    (b"mergejoy", 0x4F68FC),
    (b"nonetwork", 0x52A4B0),
    (b"inetport", 0x52AC24),
    (b"screensnapshot", 0x4F52B4),
    (b"framecounter", 0x5EB100),
    (b"noreplay", 0x5BEC7C),
    (b"replaykeys", 0x5BECA8),
    (b"logfile", 0x4E0098),
    (b"nofmv", 0x5BE870),
    (b"key", 0x4E052C),
    (b"allsecretcars", 0x758710),
    (b"secretcar", 0x758710),
    (b"usesavegamekeys", 0x755144),
    (b"sensiblesaving", 0x75AAD8),
    (b"noprecipitation", 0x71C800),
    (b"seasoncode", 0x5BECAC),
    (b"noautoneutral", 0x61416C),
    (b"nonlinearsteering", 0x4F6900),
    (b"wrreplayenable", 0x5C2CEB),
    (b"wrreplayrate", 0x5C2CEF),
    (b"alt2axcode", 0x4F68F0),
    (b"fizpatch", 0x71C6D0),
    (b"patchmaxdrawdistance", 0x71C6D4),
    (b"patchstickygrass", 0x71C6D8),
    (b"patchgravity", 0x71C6DC),
    (b"patchdamages", 0x71C6E0),
    (b"patchbrakegrip", 0x71C6E4),
    (b"patchaccelgrip", 0x71C6E8),
    (b"patchslidegrip", 0x71C6EC),
    (b"patchgripgrass", 0x71C6F0),
    (b"patchgriptarmac", 0x71C6F4),
    (b"patchgripsnow", 0x71C6F8),
    (b"patchgripmud", 0x71C6FC),
    (b"patchgripgravel", 0x71C700),
    (b"lexcontrol", 0x4E0758),
    (b"sparksmode", 0x7558DC),
];

/// The table as the image held it (before relocation): `rc2k_core::files::settings::table`
/// reads these bytes.
fn table_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    for (key, address) in ENTRIES {
        out.extend_from_slice(key);
        out.push(0);
        out.extend_from_slice(&address.to_le_bytes());
    }
    out
}

// ---- Rust version ----

/// `sub_4114FF`: the settings a text names. Converted: `rc2k_core::files::settings::table` and
/// `apply`; here the variables (the table's dwords are their addresses) are stored.
unsafe fn settings() -> Result<Vec<(Vec<u8>, u32)>, BadInput> {
    // The empty key that ends it (0x5190F1 in the image).
    let mut bytes = table_bytes();
    bytes.push(0);
    Ok(rc2k_core::files::settings::table(
        bytes.as_slice(),
        bytes.len(),
    )?)
}

pub unsafe fn apply_settings(text: *const u8) -> Result<(), BadInput> {
    if text.is_null() {
        return Err(BadInput("no settings text or no key"));
    }
    unsafe {
        let table = settings()?;
        let keys: Vec<&[u8]> = table.iter().map(|(key, _)| key.as_slice()).collect();
        // Found first, stored after: nothing is written if a key is refused.
        let found = rc2k_core::files::settings::apply(
            &GamePtr::new(text as u32).bytes("a settings text")?,
            &keys,
            &case_table(),
        )?;
        for (n, number) in found {
            // The table's (IDA) address, or where its variable lives now (`crate::moved`).
            GamePtr::new(crate::moved::place(table[n].1 as usize) as u32)
                .write(number, "a setting's variable")?;
        }
        Ok(())
    }
}

pub unsafe fn read_settings() -> Result<(), BadInput> {
    unsafe {
        match read_text(PATH.as_ptr())? {
            Some((text, _)) => apply_settings(text),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;

    #[test]
    fn table_is_the_games() {
        let bytes = super::table_bytes();
        assert_eq!(bytes, image_bytes(0x518C09, bytes.len()));
        assert_eq!(bytes.len(), 0x5190F1 - 0x518C09);
        assert_eq!(&image_bytes(0x5295CC, 12)[..], &super::PATH[..]);
        // The empty key that ends the table.
        assert_eq!(image_bytes(0x5190F1, 1), [0]);
    }

    #[test]
    fn table_reads_whole() {
        let table = unsafe { super::settings() }.unwrap();
        assert_eq!(table.len(), super::ENTRIES.len());
    }
}
