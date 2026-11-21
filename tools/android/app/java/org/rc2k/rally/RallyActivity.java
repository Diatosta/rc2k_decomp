package org.rc2k.rally;

import org.libsdl.app.SDLActivity;

/** The game's activity: SDL's, loading SDL and the game's own library (libmain.so). */
public class RallyActivity extends SDLActivity {
    @Override
    protected String[] getLibraries() {
        return new String[] {"SDL3", "main"};
    }
}
