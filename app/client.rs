use chrono::prelude::DateTime;
use cryptoxide::ed25519;
use uuid::Uuid;

use mr_wolf_tonic::clock_client::ClockClient;
use mr_wolf_tonic::{AddReq, TimeReq};

pub mod mr_wolf_tonic {
    tonic::include_proto!("mrwolf");
}

use mr_wolf::types::{mk_message, Witness};

fn mk_witness(keypair: [u8; 64], iou: u64) -> Witness {
    let nonce = Uuid::new_v4().as_bytes().clone();
    let message = mk_message(iou, &nonce);
    let sig = ed25519::signature(&message, &keypair);
    Witness { iou, nonce, sig }
}

fn mk_request(keypair: [u8; 64], id: &str, iou: u64) -> tonic::Request<TimeReq> {
    let witness = mk_witness(keypair, iou);
    tonic::Request::new(TimeReq {
        id: id.to_string(),
        iou: witness.iou,
        nonce: witness.nonce.into_iter().collect::<Vec<u8>>(),
        sig: witness.sig.into_iter().collect::<Vec<u8>>(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClockClient::connect("http://[::1]:50051").await?;
    // Subscriber generates secret and declares pub_key
    let prv_key = [0u8; 32]; // private key only for example !
    let (keypair, pub_key_arr) = ed25519::keypair(&prv_key);
    let pub_key = pub_key_arr.into_iter().collect::<Vec<u8>>();

    // Provider and subscriber agree terms.
    // Provider adds details to service
    // And hands back subscriber id
    let add_req = tonic::Request::new(AddReq {
        pot: 10000,
        pub_key,
    });

    let add_res = client.add(add_req).await?;
    // Subscriber id
    let id = add_res.into_inner().id;

    // Make a request
    let request = mk_request(keypair, &id, 1);
    let response = client.whats_the_time(request).await?;
    let start = response.into_inner().message;
    let start_time = DateTime::parse_from_str(&start, "New %+").unwrap();
    println!("ID={} UTC={:?}", id, start);
    for ii in 2..10000 {
        let request = mk_request(keypair, &id, ii);
        let response = client.whats_the_time(request).await?;
        let _utc = response.into_inner().message;
    }
    let request = mk_request(keypair, &id, 10000);
    let response = client.whats_the_time(request).await?;
    let end = response.into_inner().message;
    println!("ID={} UTC={:?}", id, end);
    let end_time = DateTime::parse_from_str(&end, "New %+").unwrap();
    println!(
        "ID={} DIFF={:?}",
        id,
        end_time.signed_duration_since(start_time)
    );
    Ok(())
}
