package org.yydcnjjw.mtool.system

import android.app.NotificationManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.MediaMetadata
import android.media.MediaMetadata.*
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.media.session.PlaybackState
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
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import org.yydcnjjw.mtool.dioxus.MainActivity


class NotificationListener : NotificationListenerService() {
    companion object {
        private const val TAG = "mtool.NotificationListener"

        private val IM_PKG_LIST = setOf("com.alibaba.android.rimet", "com.tencent.mm")

        private val NETEASE_PKG_NAME = "com.netease.cloudmusic"

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
        private external fun onNotificationPostedNative(data: String)
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

    var controller: MediaController? = null

    @RequiresApi(Build.VERSION_CODES.O)
    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        super.onNotificationPosted(sbn)

        val pkgName = sbn?.packageName ?: return

        Log.d(TAG, "onNotificationPosted: ${sbn.packageName}, ${sbn.notification.channelId}")

        if (pkgName == NETEASE_PKG_NAME) {
//            if (controller == null) {
//                (getSystemService(MEDIA_SESSION_SERVICE) as MediaSessionManager).let { manager ->
//                    controller =
//                        manager.getActiveSessions(
//                            ComponentName(
//                                this,
//                                NotificationListener::class.java
//                            )
//                        ).find { it.packageName == NETEASE_PKG_NAME }?.apply {
//                            registerCallback(object : MediaController.Callback() {
//                                override fun onPlaybackStateChanged(state: PlaybackState?) {
//                                    controller?.let {
//                                        postNative(MediaNotification(AppInfo(pkgName), it))
//                                    }
//
//                                }
//
//                                override fun onSessionDestroyed() {
//                                    super.onSessionDestroyed()
//                                    controller = null
//                                }
//                            })
//                        }
//
//                    controller?.let {
//                        postNative(MediaNotification(AppInfo(pkgName), it))
//                    }
//                }
//            }

            return
        } else if (IM_PKG_LIST.contains(pkgName)) {
            postNative(ImNotification(AppInfo(pkgName)))
        }
    }

    fun postNative(notification: Notification) {
        val data = Json.encodeToString(notification)
        Log.d(TAG, "postNative: $data")
        onNotificationPostedNative(data)
    }
}

@Serializable
data class AppInfo(
    val id: String,
)

@Serializable
sealed class Notification

@Serializable
@SerialName("Generic")
class GenericNotification(val app: AppInfo) : Notification()

@Serializable
@SerialName("Im")
class ImNotification(val app: AppInfo) : Notification()

@Serializable
enum class MediaState {
    None,
    Stopped,
    Paused,
    Playing,
    FastForwarding,
    Rewinding,
    Buffering,
    Error,
    Connecting,
    SkippingToPrevious,
    SkippingToNext,
    SkippingToQueueItem,
}

@Serializable
data class MPlaybackState(
    val state: MediaState,
    val position: Long
)

@Serializable
data class MMediaMetadata(
    val id: String,
    val title: String,
    val artist: String,
    val album: String,
)

@Serializable
@SerialName("Media")
class MediaNotification(val app: AppInfo) : Notification() {
    var state: MPlaybackState? = null
    var metadata: MMediaMetadata? = null

    constructor(app: AppInfo, controller: MediaController) : this(app) {
        controller.playbackState?.let {
            state = MPlaybackState(
                state = when (it.state) {
                    PlaybackState.STATE_NONE -> MediaState.None
                    PlaybackState.STATE_STOPPED -> MediaState.Stopped
                    PlaybackState.STATE_PAUSED -> MediaState.Paused
                    PlaybackState.STATE_PLAYING -> MediaState.Playing
                    PlaybackState.STATE_FAST_FORWARDING -> MediaState.FastForwarding
                    PlaybackState.STATE_REWINDING -> MediaState.Rewinding
                    PlaybackState.STATE_BUFFERING -> MediaState.Buffering
                    PlaybackState.STATE_ERROR -> MediaState.Error
                    PlaybackState.STATE_CONNECTING -> MediaState.Connecting
                    PlaybackState.STATE_SKIPPING_TO_PREVIOUS -> MediaState.SkippingToPrevious
                    PlaybackState.STATE_SKIPPING_TO_NEXT -> MediaState.SkippingToNext
                    PlaybackState.STATE_SKIPPING_TO_QUEUE_ITEM -> MediaState.SkippingToQueueItem
                    else -> throw Error("unreachable")
                },
                position = it.position
            )
        }

        controller.metadata?.let {
            metadata = MMediaMetadata(
                id = it.getString(METADATA_KEY_MEDIA_ID),
                title = it.getString(METADATA_KEY_TITLE),
                artist = it.getString(METADATA_KEY_ARTIST),
                album = it.getString(METADATA_KEY_ALBUM)
            )
        }
    }
}