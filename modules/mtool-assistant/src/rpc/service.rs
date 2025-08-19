use mapp::{anyhow, prelude::*, serde_json};
use mtool_system::Notification;
use pb::mtool_assistant_server::MtoolAssistantServer;
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("mtool_assistant");
}

use crate::notify::NotifyContext;

pub struct RpcService {
    ctx: Res<NotifyContext>,
}

impl RpcService {
    pub fn server(ctx: Res<NotifyContext>) -> Result<MtoolAssistantServer<Self>, anyhow::Error> {
        Ok(MtoolAssistantServer::new(Self { ctx }))
    }
}

#[tonic::async_trait]
impl pb::mtool_assistant_server::MtoolAssistant for RpcService {
    async fn post_notification(
        &self,
        request: Request<pb::Notification>,
    ) -> Result<Response<pb::Empty>, Status> {
        NotifyContext::handle_notification_posted(
            self.ctx.clone(),
            serde_json::from_slice(&request.into_inner().data)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
        )
        .await
        .map_err(|e| Status::internal(format!("{:?}", e)))?;
        Ok(Response::new(pb::Empty {}))
    }
}
