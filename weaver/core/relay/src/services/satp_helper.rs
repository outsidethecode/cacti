use config::Config;
use serde::{Serialize, Deserialize};
use sled::IVec;
use weaverpb::common::state::{RequestState, request_state};
use weaverpb::networks::networks::{NetworkAssetTransfer};
use weaverpb::relay::satp::{TransferCommenceRequest, CommenceResponseRequest};

use crate::db::Database;
use crate::error;

pub fn derive_transfer_commence_request(network_asset_transfer: NetworkAssetTransfer) -> TransferCommenceRequest {
    let session_id = "to_be_calculated_session_id"; 
    let transfer_commence_request = TransferCommenceRequest {
        message_type: "message_type1".to_string(),
        session_id: session_id.to_string(),
        transfer_context_id: "transfer_context_id1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        hash_transfer_init_claims: "hash_transfer_init_claims1".to_string(),
        hash_prev_message: "hash_prev_message1".to_string(),
        client_transfer_number: "client_transfer_number1".to_string(),
        client_signature: "client_signature1".to_string()
    };

    return transfer_commence_request;
}

pub fn create_commence_response_request() -> CommenceResponseRequest {
    let commence_response_request = CommenceResponseRequest {
        message_type: "message_type1".to_string(),
        session_id: "session_id1".to_string(),
        transfer_context_id: "transfer_context_id1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        hash_prev_message: "hash_prev_message1".to_string(),
        server_transfer_number: "server_transfer_number1".to_string(),
        server_signature: "server_signature1".to_string(),
    };

    return commence_response_request;
}

pub fn get_satp_requests_local_db(conf: Config) -> Database {
    let db = Database {
        db_path: conf.get_str("db_satp_requests_path").unwrap(),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn get_satp_requests_states_local_db(conf: Config) -> Database {
    let db = Database {
        db_path: conf.get_str("db_satp_requests_states_path").unwrap(),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn get_satp_requests_remote_db(conf: Config) -> Database {
    let db = Database {
        db_path: conf.get_str("remote_db_satp_requests_path").unwrap(),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn get_satp_requests_states_remote_db(conf: Config) -> Database {
    let db = Database {
        db_path: conf.get_str("remote_db_satp_requests_states_path").unwrap(),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn log_request_state_in_local_sapt_db(request_id: &String, target: &RequestState, conf: Config) -> Result<std::option::Option<IVec>, error::Error>{
    let db = get_satp_requests_states_local_db(conf);
    return db.set(&request_id, &target);
}

pub fn log_request_in_local_sapt_db<T: Serialize>(request_id: &String, value: T, conf: Config) -> Result<std::option::Option<IVec>, error::Error>{
    let db = get_satp_requests_local_db(conf);
    return db.set(&request_id, &value);
}

pub fn log_request_in_remote_sapt_db<T: Serialize>(request_id: &String, value: T, conf: Config) -> Result<std::option::Option<IVec>, error::Error>{
    let db = get_satp_requests_remote_db(conf);
    return db.set(&request_id, &value);
}

pub fn get_request_from_remote_sapt_db<'a, T: Deserialize<'a>>(request_id: &String, conf: Config) -> T {
    let db = get_satp_requests_remote_db(conf);
    let query: T = db.get::<T>(request_id.to_string())
    .map_err(|e| Error::GetQuery(format!("Failed to get query from db. Error: {:?}", e)))?;
}

pub fn update_request_state_in_local_satp_db(request_id: String, new_status: request_state::Status, state: Option<request_state::State>, conf: Config) {
    let db = get_satp_requests_states_local_db(conf);
    let target: RequestState = RequestState {
        status: new_status as i32,
        request_id: request_id.clone(),
        state,
    };
    db.set(&request_id, &target)
        .expect("Failed to insert into DB");
    println!("Successfully written RequestState to database");
    println!("{:?}\n", db.get::<RequestState>(request_id).unwrap())
}
