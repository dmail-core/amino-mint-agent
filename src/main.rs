use config::AppConfig;
use ic_agent::ic_types::principal;
use ic_agent::{
    agent::http_transport::ReqwestHttpReplicaV2Transport, ic_types::Principal,
    identity::Secp256k1Identity, Agent,
};
use ic_cdk::export::candid::{Decode, Encode, Nat};
use log::{error, info};
use once_cell::sync::Lazy;
use sqlx::mysql::{MySql, MySqlPoolOptions};
use sqlx::Error;
use sqlx::Pool;
use types::{
    MetadataDesc, MetadataKeyVal, MetadataPart, MetadataPurpose, MetadataVal, MintReceipt,
};

// static DB: Lazy<Pool<MySql>> = Lazy::new(|| init_db_pool().unwrap());
static CFG: Lazy<AppConfig> = Lazy::new(AppConfig::new);

pub fn init_db_pool() -> Result<Pool<MySql>, Error> {
    MySqlPoolOptions::new()
        .min_connections(15)
        .max_connections(30)
        .connect_lazy(&CFG.database_url)
}

fn create_identity() -> Secp256k1Identity {
    Secp256k1Identity::from_pem_file(&CFG.icp_config.identity_pem_path)
        .expect("Could not read the key pair.")
}

async fn mint_token(nft_ser: String, nft: Nft) {
    let transport = ReqwestHttpReplicaV2Transport::create(&CFG.icp_config.icp_domain).unwrap();

    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();

    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(create_identity())
        .build()
        .unwrap();

    //main net need to remove agent.fetch_root_key*()
    let _ = agent.fetch_root_key().await;

    let canister_id = Principal::from_text(&CFG.icp_config.agent_canister_id).unwrap();

    let metadata_desc: MetadataDesc = vec![MetadataPart {
        purpose: MetadataPurpose::Rendered,
        key_val_data: vec![
            MetadataKeyVal {
                key: "id".into(),
                val: MetadataVal::Nat64Content(nft.id.parse::<u64>().unwrap()),
            },
            MetadataKeyVal {
                key: "nft_content".into(),
                val: MetadataVal::TextContent(nft_ser.clone()),
            },
        ],
        data: vec![],
    }];

    let response = agent
        .update(&canister_id, "mintDip721")
        .with_arg(
            &Encode!(
                &Principal::from_text(nft.principal_id).unwrap(),
                &metadata_desc
            )
            .unwrap(),
        )
        .call_and_wait(waiter.clone())
        .await;
    if response.is_ok() {
        let decode_result = Decode!(response.unwrap().as_slice(), MintReceipt).unwrap();

        match decode_result {
            Ok(mint_receipt_part) => {
                println!(
                    "{}",
                    Response {
                        result: "success".to_string(),
                        message: format!("mint :{}", &nft.id),
                        token_id: Some(mint_receipt_part.id.try_into().unwrap())
                    }
                )
            }
            Err(api_error) => {
                println!(
                    "{}",
                    Response {
                        result: "fault".to_string(),
                        message: format!("mint :{}---Error:{:?}", &nft.id, api_error),
                        token_id: None
                    }
                )
            }
        }
    } else {
        let error = response.unwrap_err();
        println!("error response:{:?}", error);
    }
}

use std::env;
#[inline(always)]
fn get_args() -> (String, Nft) {
    let args: Vec<String> = env::args().collect();

    let nft = args
        .get(1)
        .unwrap_or_else(|| panic!("arg1 must not be empty"));

    (
        nft.clone(),
        serde_json::from_str(nft).expect("cid list not fount"),
    )
}

#[derive(Deserialize)]
pub struct Nft {
    principal_id: String,
    // url: String,
    id: String,
    // attributes: Vec<Attribute>,
}

// #[derive(Deserialize)]
// pub struct Attribute {
//     trait_type: String,
//     value: String,
// }

use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Debug)]
pub struct Response {
    result: String,
    message: String,
    token_id: Option<u64>,
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.token_id.is_none() {
            write!(
                f,
                r#"{{ 
    "result":"{}", 
    "message":"{}"
}}"#,
                self.result, self.message
            )
        } else {
            write!(
                f,
                r#"{{ 
    "result":"{}", 
    "message":"{}",
    "token_id":"{:?}"
}}"#,
                self.result,
                self.message,
                self.token_id.unwrap()
            )
        }
    }
}

mod config;
mod logger;
mod types;

#[tokio::main]
async fn main() {
    //log
    logger::start();

    // info!("*********************************************start********************************************************\n");

    //query nft
    let (ser_nft, nft) = get_args();
    // println!("1111111111111111111111{:?}", nft);
    mint_token(ser_nft, nft).await;

    // info!("*********************************************end**********************************************************\n");
}
