package org.rc2k.rally;

import android.app.Activity;
import android.content.Context;
import android.net.wifi.WifiManager;

/** Holds the Wi-Fi's multicast lock: without it many phones drop the multicast packets Bonjour
 *  (mDNS) finds network games with. */
public final class Multicast {
    private Multicast() {}

    private static WifiManager.MulticastLock lock;

    /** Whether the lock is held (taken the first time; {@code tag} names it). Called from a
     *  network thread of the game. */
    public static synchronized boolean hold(Activity activity, String tag) {
        try {
            if (lock == null) {
                WifiManager wifi = (WifiManager) activity.getApplicationContext()
                        .getSystemService(Context.WIFI_SERVICE);
                if (wifi == null) {
                    return false;
                }
                lock = wifi.createMulticastLock(tag);
                lock.setReferenceCounted(false);
            }
            lock.acquire();
            return true;
        } catch (RuntimeException e) {
            return false;
        }
    }
}
