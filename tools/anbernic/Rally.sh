#!/bin/bash
# The launcher's entry (Roms/APPS): the launcher has ended; it comes back when this does.
cd /mnt/mmc/Roms/APPS/rally || exit 1
[ -f /mnt/mod/ctrl/volumeCtrl.dge ] && /mnt/mod/ctrl/volumeCtrl.dge &
RC2K_PROFILE=1 ./rc2k > rc2k.out 2>&1
kill -9 $(pidof volumeCtrl.dge) 2>/dev/null
