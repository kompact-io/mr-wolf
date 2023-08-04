use chrono::prelude::{DateTime, Utc};
use mr_wolf::store::StoreError;
use tonic::{transport::Server, Code, Request, Response, Status};

use mr_wolf_tonic::clock_server::{Clock, ClockServer};
use mr_wolf_tonic::{
    AddReq, AddRes, BalanceRes, Empty, GetReq, GetRes, TimeReq, TimeRes, UpdateReq,
};
use uuid::Uuid;

pub mod mr_wolf_tonic {
    tonic::include_proto!("mrwolf"); // The string specified here must match the proto package name
}

use mr_wolf::store::Store;
use mr_wolf::types::{PubKey, Witness};

#[derive(Default)]
pub struct MyClock {
    pub store: Store,
}

type ClockResult<T> = Result<Response<T>, Status>;

fn store_error_status(err: StoreError) -> Status {
    Status::new(Code::Unknown, format!("{:?}", err))
}

#[tonic::async_trait]
impl Clock for MyClock {
    async fn add(&self, req: Request<AddReq>) -> ClockResult<AddRes> {
        let AddReq { pub_key, pot } = req.into_inner();
        let pub_key: PubKey = pub_key
            .try_into()
            .map_err(|_| Status::new(Code::InvalidArgument, "Cannot parse pub key"))?;
        let id = self
            .store
            .add(pub_key, pot)
            .map_err(|e| store_error_status(e))?
            .to_string();
        Ok(Response::new(AddRes { id }))
    }
    async fn get(&self, _req: Request<GetReq>) -> ClockResult<GetRes> {
        todo!("Not implemented error")
    }
    async fn update(&self, _req: Request<UpdateReq>) -> ClockResult<Empty> {
        todo!("Not implemented error")
    }
    async fn whats_my_balance(&self, _req: Request<Empty>) -> ClockResult<BalanceRes> {
        todo!("Not implemented error")
    }
    async fn whats_the_time(&self, req: Request<TimeReq>) -> Result<Response<TimeRes>, Status> {
        println!("Got a request from {:?}", req.remote_addr());
        let TimeReq {
            id: id_str,
            iou,
            nonce,
            sig,
        } = &req.into_inner();
        let id = Uuid::parse_str(id_str)
            .map_err(|_| Status::new(Code::InvalidArgument, "cannot parse id"))?;
        let witness = Witness::from_proto(iou, nonce, sig);
        let balance = self
            .store
            .update_balance(id, witness)
            .map_err(|_| Status::new(Code::InvalidArgument, "No entry found"))?;

        let cost = 1_u64; // Request cost
        if cost <= balance {
            let utc: DateTime<Utc> = Utc::now();
            let message = format!("New {:?}", utc);
            let reply = TimeRes { message };
            self.store
                .inc_used(id, cost)
                .map_err(|_| Status::new(Code::InvalidArgument, "No entry found"))?;
            Ok(Response::new(reply))
        } else {
            Err(Status::new(Code::ResourceExhausted, "insufficient funds"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    println!("Mr Wolf listening on {}", addr);

    let clock = MyClock::default();

    Server::builder()
        .add_service(ClockServer::new(clock))
        .serve(addr)
        .await?;

    Ok(())
}
