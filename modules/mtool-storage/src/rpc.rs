use mapp::{anyhow, prelude::*};
use mtool_rpc::Router;
use pb::mtool_storage_server::MtoolStorageServer;
use tonic::{Response, Status};

use crate::crdt::{self, CrdtService};

pub mod pb {
    tonic::include_proto!("mtool_storage");
}

pub use pb::mtool_storage_client::MtoolStorageClient;

pub struct RpcService {
    crdt: Res<CrdtService>,
}

impl RpcService {
    pub async fn setup(router: Res<Router>, crdt: Res<CrdtService>) -> Result<(), anyhow::Error> {
        router.add_service(MtoolStorageServer::new(Self { crdt }));
        Ok(())
    }
}

#[tonic::async_trait]
impl pb::mtool_storage_server::MtoolStorage for RpcService {
    type CRDTSyncStream = crdt::CrdtStream;

    async fn crdt_pull_peer_updates(
        &self,
        request: tonic::Request<pb::CrdtDocVersion>,
    ) -> Result<tonic::Response<pb::CrdtUpdates>, Status> {
        self.crdt
            .handle_pull_peer_updates(request.into_inner())
            .await
            .map(|value| Response::new(value))
            .map_err(|e| Status::internal(e.to_string()))
    }
    async fn crdt_push_local_updates(
        &self,
        request: tonic::Request<pb::CrdtUpdates>,
    ) -> Result<tonic::Response<pb::Empty>, Status> {
        self.crdt
            .handle_push_local_updates(request.into_inner())
            .await
            .map(|value| Response::new(value))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn crdt_sync(
        &self,
        request: tonic::Request<tonic::Streaming<pb::CrdtSyncMessage>>,
    ) -> Result<tonic::Response<Self::CRDTSyncStream>, Status> {
        self.crdt
            .handle_sync(request.into_inner())
            .await
            .map(|value| Response::new(value))
            .map_err(|e| Status::internal(e.to_string()))
    }
}
