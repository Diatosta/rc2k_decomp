//! Whether the already-installed program is this same build (`same_version`): the installed copy is
//! read in chunks and scanned for the version text and the copyright line.

use crate::errors::Refused;
use crate::install::installer::set_cd;
use rc2k_core::install::version::{self, Scan};

const READ_TARGET: usize = 0x100; // into the scratch
const TAIL: usize = 0xFF00;
const SCAN_LEN: u32 = version::CHUNK;
/// The scratch buffer's size: the scan zeroes and reads all of it.
const BUFFER_LEN: usize = 0x10000;

pub fn check_installed_version(
    install: &'static crate::install::owner::Install,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    install.run.same_version.set(0);
    if install.names.status.get() != 1 || install.names.source.get(0) == 0 {
        return Ok(());
    }
    // Normalise the source into the installed folder's path, and go there.
    let normalised = crate::install::normalise_path::normalised_bytes(
        &install.names.source.text(),
        &mut || crate::install::normalise_path::current_directory(),
    )?;
    let Some(installed) = normalised else {
        return Ok(());
    };
    install
        .paths
        .installed
        .write_at(0, &installed, "the installed folder")?;
    if !set_cd(&install.paths.installed) {
        return Ok(());
    }
    system.work().files.buffer.write(&[0; BUFFER_LEN]);
    // The program checks itself (the same version text is in its built-in
    // image).
    let program = crate::install::run_after_exit::RC2K;
    if let Some(file) = crate::files::handle::open_by_mode(program, 0)? {
        let buffer = &system.work().files.buffer;
        loop {
            // the previous tail carried to the front (`rep movsd`, 0x40 dwords)
            for n in 0..0x100 {
                buffer.set(n, buffer.get(TAIL + n));
            }
            let mut bytes = vec![0u8; SCAN_LEN as usize];
            let (read, ok) = crate::files::handle::read_slice(file, &mut bytes);
            buffer.write_at(READ_TARGET, &bytes[..read as usize], "the program's bytes")?;
            if !ok || read == 0 {
                break;
            }
            // The copyright is compared up to its NUL, inside its 91 bytes.
            let scanned = buffer.read_at(0, BUFFER_LEN, "the scan buffer")?;
            let copyright = system.start().texts.copyright.read();
            if let Scan::Stopped { same, .. } = version::scan(&scanned[..], &copyright[..])? {
                if same {
                    install.run.same_version.set(1);
                }
                break;
            }
        }
        crate::files::handle::close(file);
    }
    set_cd(&install.paths.game_dir);
    Ok(())
}
