use mapp::{anyhow, prelude::*};
use mtool_system::Notification;
use pb::mtool_assistant_server::MtoolAssistantServer;
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("mtool_assistant");
}

use crate::notify::{NotifyContext, NotifyMode};

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
            Notification {
                package_name: request.into_inner().package_name,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("{:?}", e)))?;
        Ok(Response::new(pb::Empty {}))
    }

    async fn set_notify_mode(
        &self,
        request: Request<pb::NotifyModeMessage>,
    ) -> Result<Response<pb::Empty>, Status> {
        let mode = match pb::NotifyMode::try_from(request.into_inner().mode)
            .map_err(|e| Status::invalid_argument(format!("{:?}", e)))?
        {
            pb::NotifyMode::DesktopMode => NotifyMode::Desktop,
            pb::NotifyMode::RemoteDesktopMode => NotifyMode::RemoteDesktop,
        };
        self.ctx.set_notify_mode(mode);

        Ok(Response::new(pb::Empty {}))
    }
}
