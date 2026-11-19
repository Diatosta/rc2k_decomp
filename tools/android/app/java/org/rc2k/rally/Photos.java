package org.rc2k.rally;

import android.app.Activity;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.net.Uri;
import android.os.Build;
import android.provider.MediaStore;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.io.OutputStream;

/** Puts a picture file into the gallery (Pictures/Rally Championship). */
public final class Photos {
    private Photos() {}

    /** Whether the PNG file at {@code path} is now in the gallery. Android 10 and later only
     *  (older versions would need the storage permission). Called from the game's thread. */
    public static boolean save(Activity activity, String path) {
        if (Build.VERSION.SDK_INT < 29) {
            return false;
        }
        ContentResolver resolver = activity.getContentResolver();
        File file = new File(path);
        ContentValues values = new ContentValues();
        values.put(MediaStore.MediaColumns.DISPLAY_NAME, file.getName());
        values.put(MediaStore.MediaColumns.MIME_TYPE, "image/png");
        values.put(MediaStore.MediaColumns.RELATIVE_PATH, "Pictures/Rally Championship");
        values.put(MediaStore.MediaColumns.IS_PENDING, 1);
        Uri uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values);
        if (uri == null) {
            return false;
        }
        try (InputStream in = new FileInputStream(file);
                OutputStream out = resolver.openOutputStream(uri)) {
            byte[] buffer = new byte[65536];
            for (int n = in.read(buffer); n >= 0; n = in.read(buffer)) {
                out.write(buffer, 0, n);
            }
            ContentValues done = new ContentValues();
            done.put(MediaStore.MediaColumns.IS_PENDING, 0);
            resolver.update(uri, done, null, null);
            return true;
        } catch (Exception e) {
            resolver.delete(uri, null, null);
            return false;
        }
    }
}
