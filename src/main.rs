use config::AppConfig;
use ic_agent::{
    agent::http_transport::ReqwestHttpReplicaV2Transport, ic_types::Principal,
    identity::BasicIdentity, Agent,
};
use ic_cdk::export::candid::{CandidType, Decode, Encode};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::Deserialize;
use sqlx::mysql::{MySql, MySqlPoolOptions};
use sqlx::Error;
use sqlx::Pool;

static DB: Lazy<Pool<MySql>> = Lazy::new(|| init_db_pool().unwrap());
static CFG: Lazy<AppConfig> = Lazy::new(AppConfig::new);

pub fn init_db_pool() -> Result<Pool<MySql>, Error> {
    MySqlPoolOptions::new()
        .min_connections(15)
        .max_connections(30)
        .connect_lazy(&CFG.database_url)
}

fn create_identity() -> BasicIdentity {
    BasicIdentity::from_pem_file(&CFG.icp_config.identity_pem_path)
        .expect("Could not read the key pair.")
}

pub type ActorResult<T> = Result<T, ErrorInfo>;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize)]
pub struct ErrorInfo {
    code: u32,
    message: String,
}

async fn mint_token(nft_vec: Vec<Nft>) {
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

    // mutil mint token
    for nft in nft_vec {
        let response = agent
            .update(&canister_id, "mint_token")
            .with_arg(
                Encode!(
                    &nft.nft_name.as_ref().unwrap(),
                    &Principal::from_text(&nft.p_id.as_ref().unwrap()).unwrap()
                )
                .unwrap(),
            )
            .call_and_wait(waiter.clone())
            .await;
        if response.is_ok() {
            let decode_result = Decode!(response.unwrap().as_slice(), ActorResult<bool>).unwrap();

            if let Ok(result) = decode_result {
                if result {
                    info!(
                        "mint :{:?}------response:{:?}\n\n",
                        &nft.nft_name.as_ref().unwrap(),
                        decode_result
                    );
                    // update status = 1
                    mint_nft_success_and_update(nft.clone()).await;
                }
            } else {
                error!(
                    "mint :{:?}------response:{:?}\n\n",
                    &nft.nft_name.as_ref().unwrap(),
                    decode_result
                );
            }
        } else {
            let error = response.unwrap_err();
            println!("error response:{:?}", error);
        }
    }
}

//sqlx
#[derive(Debug, sqlx::FromRow, Clone)]
pub struct Nft {
    p_id: Option<String>,
    nft_name: Option<String>,
}

pub async fn todo_nft() -> Vec<Nft> {
    let row = sqlx::query_as::<MySql, Nft>("select * from `nft` where `status` = 0")
        .fetch_all(&*DB)
        .await
        .unwrap();
    // dbg!(&row);
    row
}

pub async fn mint_nft_success_and_update(nft: Nft) {
    let sql = "UPDATE nft SET `status` = 1 WHERE `nft_name` = ?";
    let count = sqlx::query::<MySql>(sql)
        .bind(&nft.nft_name)
        .execute(&*DB)
        .await
        .unwrap();
    info!(
        "update {:?} status = 1------result :{:?}\n",
        nft.nft_name.unwrap(),
        count
    );
}

mod config;
mod logger;

#[tokio::main]
async fn main() {
    //log
    logger::start();
    info!("*********************************************start********************************************************\n");
    //query nft
    let row = todo_nft().await;
    //mint token
    //update status
    mint_token(row).await;
    info!("*********************************************end**********************************************************\n");
}
