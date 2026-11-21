package org.rc2k.rally;

import android.app.Activity;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.provider.MediaStore;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.io.OutputStream;

/** Opens Android's share sheet with the program's log file. */
public final class Share {
    private Share() {}

    /** Whether the share sheet was opened with the text file at {@code path}. Android 10 and
     *  later copy the file into Downloads (the media store gives it a content address the other
     *  app may read, with no file provider); older versions share the end of the text itself.
     *  Called from the game's thread. */
    public static boolean log(Activity activity, String path) {
        try {
            File file = new File(path);
            Intent send = new Intent(Intent.ACTION_SEND);
            send.setType("text/plain");
            send.putExtra(Intent.EXTRA_SUBJECT, "Rally Championship log");
            if (Build.VERSION.SDK_INT >= 29) {
                Uri uri = copyToDownloads(activity.getContentResolver(), file);
                if (uri == null) {
                    return false;
                }
                send.putExtra(Intent.EXTRA_STREAM, uri);
                send.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            } else {
                send.putExtra(Intent.EXTRA_TEXT, tail(file, 200000));
            }
            Intent chooser = Intent.createChooser(send, null);
            chooser.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            activity.startActivity(chooser);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    /** The file copied into Downloads as a new entry; null when it cannot be. */
    private static Uri copyToDownloads(ContentResolver resolver, File file) throws Exception {
        ContentValues values = new ContentValues();
        values.put(MediaStore.MediaColumns.DISPLAY_NAME, "rc2k-" + System.currentTimeMillis() + ".log");
        values.put(MediaStore.MediaColumns.MIME_TYPE, "text/plain");
        values.put(MediaStore.MediaColumns.RELATIVE_PATH, "Download");
        values.put(MediaStore.MediaColumns.IS_PENDING, 1);
        Uri uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values);
        if (uri == null) {
            return null;
        }
        try (InputStream in = new FileInputStream(file);
                OutputStream out = resolver.openOutputStream(uri)) {
            byte[] buffer = new byte[65536];
            for (int n = in.read(buffer); n >= 0; n = in.read(buffer)) {
                out.write(buffer, 0, n);
            }
        } catch (Exception e) {
            resolver.delete(uri, null, null);
            throw e;
        }
        ContentValues done = new ContentValues();
        done.put(MediaStore.MediaColumns.IS_PENDING, 0);
        resolver.update(uri, done, null, null);
        return uri;
    }

    /** The last {@code most} bytes of the file as text. */
    private static String tail(File file, int most) throws Exception {
        try (java.io.RandomAccessFile in = new java.io.RandomAccessFile(file, "r")) {
            long from = Math.max(0, in.length() - most);
            byte[] bytes = new byte[(int) (in.length() - from)];
            in.seek(from);
            in.readFully(bytes);
            return new String(bytes, "UTF-8");
        }
    }
}
