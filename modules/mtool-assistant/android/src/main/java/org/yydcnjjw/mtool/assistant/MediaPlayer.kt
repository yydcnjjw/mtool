package org.yydcnjjw.mtool.assistant

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.Handler
import android.util.Log
import androidx.core.content.ContextCompat
import androidx.core.net.toUri
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DataSpec
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.datasource.ResolvingDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.session.MediaController
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSession.ControllerInfo
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionToken


@UnstableApi
class PlaybackService : MediaSessionService() {
    companion object {
        const val TAG = "assistant.PlaybackService"

        @JvmStatic
        private external fun realUri(uri: String): String
    }

    val dataSourceFactory: DataSource.Factory =
        ResolvingDataSource.Factory(DefaultHttpDataSource.Factory()) { dataSpec: DataSpec ->
            dataSpec.withUri(realUri(dataSpec.uri.toString()).toUri())
        }

    private var mediaSession: MediaSession? = null

    override fun onCreate() {
        super.onCreate()
        val player = ExoPlayer.Builder(this)
            .setName("Mtool")
            .setMediaSourceFactory(
                DefaultMediaSourceFactory(this).setDataSourceFactory(
                    dataSourceFactory
                )
            )
            .setDeviceVolumeControlEnabled(true)
            .setHandleAudioBecomingNoisy(true)
            .build()
        mediaSession = MediaSession.Builder(this, player).build()
    }

    override fun onGetSession(controllerInfo: ControllerInfo): MediaSession? = mediaSession

    override fun onDestroy() {
        mediaSession?.run {
            player.release()
            release()
            mediaSession = null
        }
        super.onDestroy()
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
        pauseAllPlayersAndStopSelf()
    }
}


@UnstableApi
class PlaybackController(
    val controller: MediaController
) {
    val handler = Handler(controller.applicationLooper)

    companion object {
        const val TAG = "assistant.PlaybackController"

        @JvmStatic
        fun connect(context: Context, callback: Callback) {
            runCatching {
                val sessionToken =
                    SessionToken(context, ComponentName(context, PlaybackService::class.java))
                val controllerFuture = MediaController.Builder(context, sessionToken)
                    .buildAsync()
                controllerFuture.addListener({
                    Log.d(TAG, "controller is connected")
                    callback.invoke(PlaybackController(controllerFuture.get()))
                }, ContextCompat.getMainExecutor(context))
            }.onFailure {
                Log.d(TAG, "$it")
            }
        }
    }

    init {
        controller.run {
            repeatMode = Player.REPEAT_MODE_ALL
            shuffleModeEnabled = true
        }
    }

    fun play() {
        handler.postAtFrontOfQueue {
            controller.play()
        }
    }

    fun pause() {
        handler.postAtFrontOfQueue {
            controller.pause()
        }
    }

    var volume: Float = 1f

    fun addMediaItems(playlist: Array<String>) {
        handler.postAtFrontOfQueue {
            controller.run {
                addMediaItems(playlist.map { uri -> MediaItem.fromUri(uri) })
                prepare()

                Log.d(TAG, "current media count: $mediaItemCount")
            }
        }
    }
}

@UnstableApi
class Callback(var handle: Long = 0) {
    fun invoke(value: PlaybackController) {
        if (handle == 0.toLong()) {
            return
        }
        invokeNative(handle, value)
        handle = 0
    }

    companion object {
        @JvmStatic
        private external fun invokeNative(handle: Long, value: PlaybackController)
    }
}