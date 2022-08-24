use ic_cdk::export::candid::{CandidType, Nat};
use serde::{Deserialize, Serialize};

pub type MintReceipt = Result<MintReceiptPart, ApiError>;

#[derive(CandidType, Debug, Deserialize)]
pub enum ApiError {
    Unauthorized,
    InvalidTokenId,
    ZeroAddress,
    Other,
    AliasFormatFail(String),
    AliasHasBeenTaken,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct MintReceiptPart {
    pub token_id: u64,
    pub id: u128,
}

pub type MetadataDesc = Vec<MetadataPart>;

#[derive(CandidType, Clone, Deserialize)]
pub struct MetadataPart {
    pub purpose: MetadataPurpose,
    pub key_val_data: Vec<MetadataKeyVal>,
    pub data: Vec<u8>,
}

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub enum MetadataPurpose {
    Preview,
    Rendered,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct MetadataKeyVal {
    pub key: String,
    pub val: MetadataVal,
}

#[derive(CandidType, Clone, Deserialize, PartialEq)]
pub enum MetadataVal {
    TextContent(String),
    BlobContent(Vec<u8>),
    NatContent(Nat),
    Nat8Content(u8),
    Nat16Content(u16),
    Nat32Content(u32),
    Nat64Content(u64),
}
