use std::{pin::Pin, sync::Arc};

use mapp::{
    anyhow,
    futures::{Stream, StreamExt},
    prelude::*,
    sync::Mutex,
    tokio::sync::{broadcast, Mutex as AsyncMutex},
    tracing::warn,
};
use pb::{media_player_client, media_player_server, player_event::Event, Empty, MediaMetadata};

use super::{MediaItem, Player, PlayerEvent, PlayerEventStream};

pub mod pb {
    tonic::include_proto!("mtool_assistant.media");
}

pub type AnyPlayer = Arc<dyn Player + Send + Sync>;

pub struct PlayerService {
    player: Mutex<Option<AnyPlayer>>,
}

impl PlayerService {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {
            player: Mutex::new(None),
        }))
    }

    pub fn server(
        this: Res<Self>,
    ) -> Result<media_player_server::MediaPlayerServer<Res<Self>>, anyhow::Error> {
        Ok(media_player_server::MediaPlayerServer::new(this))
    }

    pub fn set_player(&self, player: AnyPlayer) {
        *self.player.lock() = Some(player);
    }

    fn player(&self) -> Result<AnyPlayer, tonic::Status> {
        Ok(self
            .player
            .lock()
            .clone()
            .ok_or_else(|| tonic::Status::unavailable("player is not ready"))?
            .to_owned())
    }
}

#[tonic::async_trait]
impl media_player_server::MediaPlayer for Res<PlayerService> {
    async fn play(
        &self,
        _request: tonic::Request<pb::Empty>,
    ) -> Result<tonic::Response<pb::Empty>, tonic::Status> {
        self.player()?
            .play()
            .await
            .map_err(|e| tonic::Status::internal(format!("{e:?}")))?;
        Ok(tonic::Response::new(Empty {}))
    }

    async fn pause(
        &self,
        _request: tonic::Request<pb::Empty>,
    ) -> Result<tonic::Response<pb::Empty>, tonic::Status> {
        self.player()?
            .pause()
            .await
            .map_err(|e| tonic::Status::internal(format!("{e:?}")))?;
        Ok(tonic::Response::new(Empty {}))
    }

    async fn get_volume(
        &self,
        _request: tonic::Request<pb::Empty>,
    ) -> Result<tonic::Response<pb::Volume>, tonic::Status> {
        let volume = self
            .player()?
            .volume()
            .await
            .map_err(|e| tonic::Status::internal(format!("{e:?}")))?;
        Ok(tonic::Response::new(pb::Volume { value: volume }))
    }

    async fn set_volume(
        &self,
        request: tonic::Request<pb::Volume>,
    ) -> Result<tonic::Response<pb::Empty>, tonic::Status> {
        self.player()?
            .set_volume(request.into_inner().value)
            .await
            .map_err(|e| tonic::Status::internal(format!("{e:?}")))?;
        Ok(tonic::Response::new(Empty {}))
    }

    async fn add_media_items(
        &self,
        request: tonic::Request<pb::MediaItemList>,
    ) -> Result<tonic::Response<pb::Empty>, tonic::Status> {
        self.player()?
            .add_media_items(
                request
                    .into_inner()
                    .items
                    .into_iter()
                    .map(|item| MediaItem::new(item.url))
                    .collect(),
            )
            .await
            .map_err(|e| tonic::Status::internal(format!("{e:?}")))?;
        Ok(tonic::Response::new(Empty {}))
    }

    type ListenStream = Pin<Box<dyn Stream<Item = Result<pb::PlayerEvent, tonic::Status>> + Send>>;

    async fn listen(
        &self,
        _request: tonic::Request<pb::Empty>,
    ) -> Result<tonic::Response<Self::ListenStream>, tonic::Status> {
        let stream = self
            .player()?
            .listen()
            .await
            .map_err(|e| tonic::Status::internal(format!("{e:?}")))?;

        Ok(tonic::Response::new(Box::pin(stream.map(|ev| match ev {
            Ok(ev) => Ok(pb::PlayerEvent {
                event: Some(match ev {
                    PlayerEvent::MediaMetadataChanged(metadata) => {
                        Event::MediaMetadataChanged(pb::MediaMetadata { id: metadata.id })
                    }
                }),
            }),
            Err(e) => Err(tonic::Status::internal(format!("{e:?}"))),
        }))))
    }
}

pub struct RemotePlayer {
    client: AsyncMutex<media_player_client::MediaPlayerClient<tonic::transport::Channel>>,
}

impl RemotePlayer {
    pub async fn connect(address: String) -> Result<Self, anyhow::Error> {
        Ok(Self {
            client: AsyncMutex::new(
                media_player_client::MediaPlayerClient::connect(address).await?,
            ),
        })
    }
}

#[async_trait]
impl Player for RemotePlayer {
    async fn play(&self) -> Result<(), anyhow::Error> {
        self.client
            .lock()
            .await
            .play(tonic::Request::new(Empty {}))
            .await?;
        Ok(())
    }

    async fn pause(&self) -> Result<(), anyhow::Error> {
        self.client
            .lock()
            .await
            .pause(tonic::Request::new(Empty {}))
            .await?;
        Ok(())
    }

    async fn volume(&self) -> Result<f64, anyhow::Error> {
        Ok(self
            .client
            .lock()
            .await
            .get_volume(tonic::Request::new(Empty {}))
            .await?
            .into_inner()
            .value)
    }

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error> {
        self.client
            .lock()
            .await
            .set_volume(tonic::Request::new(pb::Volume { value }))
            .await?;
        Ok(())
    }

    async fn add_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error> {
        self.client
            .lock()
            .await
            .add_media_items(tonic::Request::new(pb::MediaItemList {
                items: items
                    .into_iter()
                    .map(|item| pb::MediaItem { url: item.uri })
                    .collect(),
            }))
            .await?;
        Ok(())
    }

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error> {
        Ok(Box::pin(
            self.client
                .lock()
                .await
                .listen(tonic::Request::new(Empty {}))
                .await?
                .into_inner()
                .map(|ev| match ev {
                    Ok(pb::PlayerEvent { event }) => match event {
                        Some(ev) => Ok(match ev {
                            pb::player_event::Event::MediaMetadataChanged(MediaMetadata { id }) => {
                                PlayerEvent::MediaMetadataChanged(super::MediaMetadata {
                                    id,
                                    ..Default::default()
                                })
                            }
                        }),
                        None => Err(broadcast::error::RecvError::Closed),
                    },
                    Err(e) => {
                        warn!("{e:?}");
                        Err(broadcast::error::RecvError::Closed)
                    }
                }),
        ))
    }
}
