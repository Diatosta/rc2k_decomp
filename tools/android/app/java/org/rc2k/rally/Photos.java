package org.rc2k.rally;

import android.app.Activity;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.pm.PackageManager;
import android.media.MediaScannerConnection;
import android.net.Uri;
import android.os.Build;
import android.os.Environment;
import android.provider.MediaStore;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.io.OutputStream;

/** Puts a picture file into the gallery, in the album "Rally Championship" (the gallery apps list
 *  the folder Pictures/Rally Championship as an album of that name). */
public final class Photos {
    private Photos() {}

    /** Whether the PNG file at {@code path} is now in the gallery. Android 10 and later insert it
     *  into the media store; older versions copy it into the public Pictures folder, which needs the
     *  storage permission (asked the first time, so the first picture is skipped). Called from the
     *  game's thread. */
    public static boolean save(Activity activity, String path) {
        if (Build.VERSION.SDK_INT < 29) {
            return saveLegacy(activity, path);
        }
        ContentResolver resolver = activity.getContentResolver();
        File file = new File(path);
        ContentValues values = new ContentValues();
        values.put(MediaStore.MediaColumns.DISPLAY_NAME, file.getName());
        values.put(MediaStore.MediaColumns.MIME_TYPE, "image/png");
        values.put(MediaStore.MediaColumns.RELATIVE_PATH, "Pictures/Rally Championship");
        values.put(MediaStore.MediaColumns.IS_PENDING, 1);
        Uri uri;
        try {
            uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values);
        } catch (RuntimeException e) {
            // A failed insert must not leave the game's thread with a Java exception pending.
            return false;
        }
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

    /** Android 9 and older: copies the file into Pictures/Rally Championship and has it scanned,
     *  so the gallery lists it. False without the storage permission (which it asks for). */
    private static boolean saveLegacy(Activity activity, String path) {
        String permission = "android.permission.WRITE_EXTERNAL_STORAGE";
        if (activity.checkSelfPermission(permission) != PackageManager.PERMISSION_GRANTED) {
            activity.runOnUiThread(() -> activity.requestPermissions(new String[] {permission}, 1));
            return false;
        }
        File folder =
                new File(
                        Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES),
                        "Rally Championship");
        File source = new File(path);
        File target = new File(folder, source.getName());
        if (!folder.isDirectory() && !folder.mkdirs()) {
            return false;
        }
        try (InputStream in = new FileInputStream(source);
                OutputStream out = new java.io.FileOutputStream(target)) {
            byte[] buffer = new byte[65536];
            for (int n = in.read(buffer); n >= 0; n = in.read(buffer)) {
                out.write(buffer, 0, n);
            }
        } catch (Exception e) {
            return false;
        }
        MediaScannerConnection.scanFile(
                activity, new String[] {target.getPath()}, new String[] {"image/png"}, null);
        return true;
    }
}
