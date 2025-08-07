package org.yydcnjjw.mtool.system

import android.app.NotificationManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.provider.Settings
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log
import androidx.annotation.RequiresApi
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.ServiceCompat
import org.yydcnjjw.mtool.dioxus.MainActivity

class NotificationListener : NotificationListenerService() {
    companion object {
        private const val TAG = "mtool.NotificationListener"
        private fun requestRebind(context: Context) {
            requestRebind(
                ComponentName(
                    context,
                    NotificationListener::class.java
                )
            )
        }

        fun isPermEnabled(context: Context): Boolean {
            val flat = Settings.Secure.getString(
                context.contentResolver,
                "enabled_notification_listeners"
            )
            return flat.contains(context.packageName)
        }

        @RequiresApi(Build.VERSION_CODES.O)
        fun startService(context: Context) {
            Log.d(TAG, "start service")
            val intent = Intent(context, NotificationListener::class.java)
            context.startForegroundService(intent)
        }

        @RequiresApi(Build.VERSION_CODES.O)
        @JvmStatic
        fun start(activity: MainActivity) {
            Log.d(TAG, "start")

            activity.runOnUiThread {
                if (isPermEnabled(activity)) {
                    return@runOnUiThread startService(activity)
                }
                activity.launchForActivityResult(
                    Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS)
                ) { it ->
                    startService(activity)
                }
            }
        }


        init {
            System.loadLibrary("dioxusmain")
        }

        @JvmStatic
        private external fun onNotificationPostedNative(packageName: String, channelId: String)
    }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "onCreate")
        startForeground()
        // requestRebind(this)
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "onDestroy")
    }

    fun startForeground() {
        val channel = NotificationChannelCompat.Builder(
            "org.yydcnjjw.mtool.notificationListener",
            NotificationManager.IMPORTANCE_DEFAULT
        ).setName("notify").build()

        NotificationManagerCompat.from(this).createNotificationChannel(channel)

        ServiceCompat.startForeground(
            this,
            100,
            NotificationCompat.Builder(this, channel.id)
                // .setSmallIcon(R.drawable.ic_launcher_background)
                .setContentTitle("Mtool")
                .build(),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK
            } else {
                0
            }
        )
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "onStartCommand")
        startForeground()
        return START_STICKY
    }

    override fun onListenerConnected() {
        super.onListenerConnected()
        Log.d(TAG, "onListenerConnected")
    }

    override fun onListenerDisconnected() {
        super.onListenerDisconnected()
        Log.d(TAG, "onListenerDisconnected")
        requestRebind(this)
    }

    @RequiresApi(Build.VERSION_CODES.O)
    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        super.onNotificationPosted(sbn)
        sbn ?: return

        Log.d(TAG, "onNotificationPosted: ${sbn.packageName}, ${sbn.notification.channelId}")
        onNotificationPostedNative(sbn.packageName, sbn.notification.channelId)
    }
}