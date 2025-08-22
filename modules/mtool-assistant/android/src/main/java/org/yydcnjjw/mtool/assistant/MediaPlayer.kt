package org.yydcnjjw.mtool.assistant

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.Handler
import android.util.Log
import androidx.core.content.ContextCompat
import androidx.core.net.toUri
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.text.CueGroup
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.ResolvingDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.exoplayer.util.EventLogger
import androidx.media3.session.MediaController
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSession.ControllerInfo
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionToken
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@UnstableApi
class PlaybackService : MediaSessionService() {
    companion object {
        const val TAG = "assistant.PlaybackService"

        @JvmStatic
        private external fun resolveUri(uri: String): String
    }

    private var mediaSession: MediaSession? = null

    override fun onCreate() {
        super.onCreate()
        val player = ExoPlayer.Builder(this)
            .setName("Mtool")
            .setMediaSourceFactory(
                DefaultMediaSourceFactory(this).setDataSourceFactory(
                    ResolvingDataSource.Factory(DefaultDataSource.Factory(this)) {
                        it.withUri(resolveUri(it.uri.toString()).toUri())
                    }
                )
            )
            .setDeviceVolumeControlEnabled(true)
            .setHandleAudioBecomingNoisy(true)
            .build()

        player.addAnalyticsListener(EventLogger())

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
        fun connect(context: Context, handler: OnceCallback) {
            runCatching {
                val sessionToken =
                    SessionToken(context, ComponentName(context, PlaybackService::class.java))
                val controllerFuture = MediaController.Builder(context, sessionToken)
                    .buildAsync()
                controllerFuture.addListener({
                    Log.d(TAG, "controller is connected")
                    handler.invoke(PlaybackController(controllerFuture.get()))
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

    fun setMediaItems(items: Array<String>) {
        handler.postAtFrontOfQueue {
            controller.run {
                setMediaItems(items.map { data ->
                    Json.decodeFromString<MtoolMediaItem>(data).run {
                        val builder = MediaItem.Builder()
                            .setMediaId(id)
                            .setUri(sourceUri)
                            .setSubtitleConfigurations(timedMetadataSourceUriList.map {
                                MediaItem.SubtitleConfiguration.Builder(it.toUri())
                                    .setMimeType(MimeTypes.TEXT_VTT)
                                    .setSelectionFlags(C.SELECTION_FLAG_FORCED)
                                    .build()
                            })
                        metadata?.run {
                            builder.setMediaMetadata(
                                MediaMetadata.Builder()
                                    .setMediaType(MediaMetadata.MEDIA_TYPE_MUSIC)
                                    .setTitle(title)
                                    .setArtist(artist)
                                    .setAlbumTitle(album)
                                    .setArtworkUri(picUrl.toUri())
                                    .setDurationMs(duration.toLong()).build()
                            )
                        }
                        builder.build()
                    }
                }.shuffled())
                prepare()

                Log.d(TAG, "current media count: $mediaItemCount")
            }
        }
    }

    fun currentMediaItem(callback: OnceCallback) {
        handler.postAtFrontOfQueue {
            callback.invoke(controller.currentMediaItem?.localConfiguration?.uri?.toString() ?: "")
        }
    }

    var listener: Player.Listener? = null

    fun listen(cb: Callback) {
        listener?.let { controller.removeListener(it) }

        listener = object : Player.Listener {
            override fun onMediaMetadataChanged(mediaMetadata: MediaMetadata) {
                val data = Json.encodeToString<PlayerEvent>(
                    MediaMetadataChangedEvent(
                        MtoolMediaMetadata(
                            title = mediaMetadata.title.toString(),
                            artist = mediaMetadata.artist.toString(),
                            album = mediaMetadata.albumTitle.toString(),
                            picUrl = mediaMetadata.artworkUri.toString(),
                            duration = mediaMetadata.durationMs?.toUInt() ?: 0u,
                        )
                    )
                )
                Log.d(TAG, "listen: $data")
                cb.invoke(data)
            }

            override fun onCues(cueGroup: CueGroup) {
                // cueGroup.
                Log.d(TAG, "onCues: $cueGroup")
            }

            override fun onPlayerError(error: PlaybackException) {
                Log.d(TAG, "player error: $error")
            }
        }

        listener?.let {
            controller.addListener(it)
        }

    }
}

@Serializable
data class MtoolMediaItem(
    val id: String,
    @SerialName("source_uri")
    val sourceUri: String,
    @SerialName("timed_metadata_source_uri_list")
    val timedMetadataSourceUriList: List<String>,
    val metadata: MtoolMediaMetadata?
)

@Serializable
data class MtoolMediaMetadata(
    val title: String,
    val artist: String,
    val album: String,
    @SerialName("pic_url")
    val picUrl: String,
    val duration: UInt
)

@Serializable
sealed class PlayerEvent

@Serializable
@SerialName("MediaMetadataChanged")
class MediaMetadataChangedEvent(val metadata: MtoolMediaMetadata) : PlayerEvent()

@Serializable
@SerialName("TimedCuesChanged")
class TimedCuesChangedEvent(
    @SerialName("track_id")
    val trackId: String,
) : PlayerEvent()

class Callback(var handle: Long = 0) {
    fun invoke(vararg args: Any) {
        assert(handle != 0L)
        invokeNative(handle, args)
    }

    companion object {
        @JvmStatic
        private external fun invokeNative(handle: Long, args: Array<out Any>)
    }
}

class OnceCallback(var handle: Long = 0) {
    fun invoke(vararg args: Any) {
        assert(handle != 0L)
        invokeNative(handle, args)
        handle = 0
    }

    companion object {
        @JvmStatic
        private external fun invokeNative(handle: Long, args: Array<out Any>)
    }
}
