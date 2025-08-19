use mapp::{anyhow, prelude::*, serde_json};
use pb::notify_server::NotifyServer;
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("mtool_assistant.notify");
}

pub(crate) use pb::notify_client::NotifyClient;
pub(crate) use pb::*;

use super::receiver::NotifyReceiver;

pub struct RpcService {
    receiver: Res<NotifyReceiver>,
}

impl RpcService {
    pub fn server(ctx: Res<NotifyReceiver>) -> Result<NotifyServer<Self>, anyhow::Error> {
        Ok(NotifyServer::new(Self { receiver: ctx }))
    }
}

#[tonic::async_trait]
impl pb::notify_server::Notify for RpcService {
    async fn post_notification(
        &self,
        request: Request<pb::Notification>,
    ) -> Result<Response<pb::Empty>, Status> {
        NotifyReceiver::handle_notification_posted(
            self.receiver.clone(),
            serde_json::from_slice(&request.into_inner().data)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
        )
        .await
        .map_err(|e| Status::internal(format!("{:?}", e)))?;
        Ok(Response::new(pb::Empty {}))
    }
}
