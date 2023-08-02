use config::Config;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::IVec;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::Response;
use weaverpb::common::ack::{ack, Ack};
use weaverpb::common::state::{request_state, RequestState};
use weaverpb::networks::networks::NetworkAssetTransfer;
use weaverpb::relay::satp::satp_client::SatpClient;
use weaverpb::relay::satp::{
    AckCommenceRequest, LockAssertionRequest, TransferCommenceRequest,
    TransferProposalClaimsRequest, TransferProposalReceiptRequest, LockAssertionReceiptRequest,
};

use crate::db::Database;
use crate::error::{self, Error};
use crate::relay_proto::LocationSegment;

use super::constants::{
    SATP_DB_PATH, SATP_REMOTE_REQUESTS_DB_PATH, SATP_REMOTE_REQUESTS_STATES_DB_PATH,
    SATP_REQUESTS_DB_PATH, SATP_REQUESTS_STATES_DB_PATH,
};

// Sends a request to the receiving gateway
pub fn spawn_send_transfer_proposal_claims_request(
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    relay_tlsca_cert_path: String,
    conf: Config,
) {
    println!(
        "Sending transfer proposal claims request to receiver gateway: {:?}:{:?}",
        relay_host, relay_port
    );
    // Spawning new thread to make the call_transfer_proposal_claims to receiver gateway
    tokio::spawn(async move {
        let request_id =
            get_request_id_from_transfer_proposal_claims(transfer_proposal_claims_request.clone());
        println!(
            "Sending transfer proposal claims request to receiver gateway: Request ID = {:?}",
            request_id
        );
        let result = call_transfer_proposal_claims(
            relay_host,
            relay_port,
            use_tls,
            relay_tlsca_cert_path.to_string(),
            transfer_proposal_claims_request.clone(),
        )
        .await;

        println!("Received Ack from sending gateway: {:?}\n", result);
        // Updates the request in the DB depending on the response status from the sending gateway
        log_request_result_in_local_satp_db(&request_id, result, conf);
    });
}

// Sends a request to the receiving gateway
pub fn spawn_send_transfer_commence_request(
    transfer_commence_request: TransferCommenceRequest,
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    relay_tlsca_cert_path: String,
    conf: Config,
) {
    println!(
        "Sending transfer commence request to receiver gateway: {:?}:{:?}",
        relay_host, relay_port
    );
    // Spawning new thread to make the call_transfer_commence to receiver gateway
    tokio::spawn(async move {
        let request_id = transfer_commence_request.session_id.to_string();
        println!(
            "Sending transfer commence request to receiver gateway: Request ID = {:?}",
            request_id
        );
        let result = call_transfer_commence(
            relay_host,
            relay_port,
            use_tls,
            relay_tlsca_cert_path.to_string(),
            transfer_commence_request.clone(),
        )
        .await;

        println!("Received Ack from sending gateway: {:?}\n", result);
        // Updates the request in the DB depending on the response status from the sending gateway
        log_request_result_in_local_satp_db(&request_id, result, conf);
    });
}

pub fn spawn_send_transfer_proposal_receipt_request(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    relay_tlsca_cert_path: String,
    conf: Config,
) {
    tokio::spawn(async move {
        let request_id = get_request_id_from_transfer_proposal_receipt(
            transfer_proposal_receipt_request.clone(),
        );
        println!(
            "Sending transfer proposal receipt back to sending gateway: Request ID = {:?}",
            request_id
        );
        let result = call_transfer_proposal_receipt(
            relay_host,
            relay_port,
            use_tls,
            relay_tlsca_cert_path.to_string(),
            transfer_proposal_receipt_request.clone(),
        )
        .await;

        println!("Received Ack from sending gateway: {:?}\n", result);
        // Updates the request in the DB depending on the response status from the sending gateway
        log_request_result_in_local_satp_db(&request_id, result, conf);
    });
}

pub fn spawn_send_ack_commence_request(
    ack_commence_request: AckCommenceRequest,
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    relay_tlsca_cert_path: String,
    conf: Config,
) {
    tokio::spawn(async move {
        let request_id = ack_commence_request.session_id.to_string();
        println!(
            "Sending commence response back to sending gateway: Request ID = {:?}",
            request_id
        );
        let result = call_ack_commence(
            relay_host,
            relay_port,
            use_tls,
            relay_tlsca_cert_path.to_string(),
            ack_commence_request.clone(),
        )
        .await;

        println!("Received Ack from sending gateway: {:?}\n", result);
        // Updates the request in the DB depending on the response status from the sending gateway
        log_request_result_in_local_satp_db(&request_id, result, conf);
    });
}

pub fn spawn_send_perform_lock_request(
    lock_assertion_request: LockAssertionRequest,
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    relay_tlsca_cert_path: String,
    conf: Config,
) {
    tokio::spawn(async move {
        println!(
            "Locking the asset of the lock assertion request {:?}",
            lock_assertion_request
        );
        // TODO
        // Call the driver to check the asset status
        // Subscribe to the status event
        // Once the asset is locked, call the lock_assertion endpoint
        // log the results
        let result = call_lock_assertion(
            relay_host,
            relay_port,
            use_tls,
            relay_tlsca_cert_path.to_string(),
            lock_assertion_request.clone(),
        )
        .await;

        println!("Received Ack from sending gateway: {:?}\n", result);
        // Updates the request in the DB depending on the response status from the sending gateway
        let request_id = lock_assertion_request.session_id.to_string();
        log_request_result_in_local_satp_db(&request_id, result, conf);
    });
}

pub fn spawn_send_lock_assertion_receipt_request(
    lock_assertion_receipt_request: LockAssertionReceiptRequest,
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    relay_tlsca_cert_path: String,
    conf: Config,
) {
    tokio::spawn(async move {
        let request_id = lock_assertion_receipt_request.session_id.to_string();
        println!(
            "Sending lock assertion receipt back to sending gateway: Request ID = {:?}",
            request_id
        );
        let result = call_lock_assertion_receipt(
            relay_host,
            relay_port,
            use_tls,
            relay_tlsca_cert_path.to_string(),
            lock_assertion_receipt_request.clone(),
        )
        .await;

        println!("Received Ack from sending gateway: {:?}\n", result);
        // Updates the request in the DB depending on the response status from the sending gateway
        log_request_result_in_local_satp_db(&request_id, result, conf);
    });
}

// Call the transfer_commence endpoint on the receiver gateway
pub async fn call_transfer_commence(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
    transfer_commence_request: TransferCommenceRequest,
) -> Result<Response<Ack>, Box<dyn std::error::Error>> {
    let mut satp_client: SatpClient<Channel> =
        create_satp_client(relay_host, relay_port, use_tls, tlsca_cert_path).await?;
    println!(
        "Sending the transfer commence request: {:?}",
        transfer_commence_request.clone()
    );
    let response = satp_client
        .transfer_commence(transfer_commence_request.clone())
        .await?;
    Ok(response)
}

// Call the call_transfer_proposal_claims endpoint on the receiver gateway
pub async fn call_transfer_proposal_claims(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
) -> Result<Response<Ack>, Box<dyn std::error::Error>> {
    let mut satp_client: SatpClient<Channel> =
        create_satp_client(relay_host, relay_port, use_tls, tlsca_cert_path).await?;
    println!(
        "Sending the transfer proposal claims request: {:?}",
        transfer_proposal_claims_request.clone()
    );
    let response = satp_client
        .transfer_proposal_claims(transfer_proposal_claims_request.clone())
        .await?;
    Ok(response)
}

// Call the call_transfer_proposal_receipt endpoint on the sending gateway
pub async fn call_transfer_proposal_receipt(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
) -> Result<Response<Ack>, Box<dyn std::error::Error>> {
    let mut satp_client: SatpClient<Channel> =
        create_satp_client(relay_host, relay_port, use_tls, tlsca_cert_path).await?;
    println!(
        "Sending the transfer proposal receipt request: {:?}",
        transfer_proposal_receipt_request.clone()
    );
    let response = satp_client
        .transfer_proposal_receipt(transfer_proposal_receipt_request.clone())
        .await?;
    Ok(response)
}

// Call the ack_commence endpoint on the sending gateway
pub async fn call_ack_commence(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
    ack_commence_request: AckCommenceRequest,
) -> Result<Response<Ack>, Box<dyn std::error::Error>> {
    let mut satp_client: SatpClient<Channel> =
        create_satp_client(relay_host, relay_port, use_tls, tlsca_cert_path).await?;
    println!(
        "Sending the commence response request: {:?}",
        ack_commence_request.clone()
    );
    let response = satp_client
        .ack_commence(ack_commence_request.clone())
        .await?;
    Ok(response)
}

// Call the lock_assertion endpoint on the sending gateway
pub async fn call_lock_assertion(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
    lock_assertion_request: LockAssertionRequest,
) -> Result<Response<Ack>, Box<dyn std::error::Error>> {
    let mut satp_client: SatpClient<Channel> =
        create_satp_client(relay_host, relay_port, use_tls, tlsca_cert_path).await?;
    println!(
        "Sending the lock assertion request: {:?}",
        lock_assertion_request.clone()
    );
    let response = satp_client
        .lock_assertion(lock_assertion_request.clone())
        .await?;
    Ok(response)
}

// Call the ack_commence endpoint on the sending gateway
pub async fn call_lock_assertion_receipt(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
    lock_assertion_receipt_request: LockAssertionReceiptRequest,
) -> Result<Response<Ack>, Box<dyn std::error::Error>> {
    let mut satp_client: SatpClient<Channel> =
        create_satp_client(relay_host, relay_port, use_tls, tlsca_cert_path).await?;
    println!(
        "Sending the lock assertion receipt request: {:?}",
        lock_assertion_receipt_request.clone()
    );
    let response = satp_client
        .lock_assertion_receipt(lock_assertion_receipt_request.clone())
        .await?;
    Ok(response)
}

pub fn log_request_result_in_local_satp_db(
    request_id: &String,
    result: Result<Response<Ack>, Box<dyn std::error::Error>>,
    conf: Config,
) {
    match result {
        Ok(ack_response) => {
            let ack_response_into_inner = ack_response.into_inner().clone();
            // This match first checks if the status is valid.
            match ack::Status::from_i32(ack_response_into_inner.status) {
                Some(status) => match status {
                    ack::Status::Ok => update_request_state_in_local_satp_db(
                        request_id.to_string(),
                        request_state::Status::Pending,
                        None,
                        conf,
                    ),
                    ack::Status::Error => update_request_state_in_local_satp_db(
                        request_id.to_string(),
                        request_state::Status::Error,
                        Some(request_state::State::Error(
                            ack_response_into_inner.message.to_string(),
                        )),
                        conf,
                    ),
                },
                None => update_request_state_in_local_satp_db(
                    request_id.to_string(),
                    request_state::Status::Error,
                    Some(request_state::State::Error(
                        "Status is not supported or is invalid".to_string(),
                    )),
                    conf,
                ),
            }
        }
        Err(result_error) => update_request_state_in_local_satp_db(
            request_id.to_string(),
            request_state::Status::Error,
            Some(request_state::State::Error(format!("{:?}", result_error))),
            conf,
        ),
    }
}

pub fn create_ack_error_message(
    request_id: String,
    error_message: String,
    e: Error,
) -> Result<tonic::Response<Ack>, tonic::Status> {
    println!("{}: {}", error_message, request_id);
    let reply: Result<Response<Ack>, tonic::Status> = Ok(Response::new(Ack {
        status: ack::Status::Error as i32,
        request_id: request_id,
        message: format!("{} {:?}", error_message, e),
    }));
    println!("Sending Ack back with an error: {:?}\n", reply);
    return reply;
}

pub fn create_transfer_proposal_claims_request(
    network_asset_transfer: NetworkAssetTransfer,
) -> TransferProposalClaimsRequest {
    let session_id = "to_be_calculated_session_id";
    let transfer_proposal_claims_request = TransferProposalClaimsRequest {
        message_type: "message_type1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        asset_asset_id: "asset_asset_id".to_string(),
        asset_profile_id: "asset_profile_id".to_string(),
        verified_originator_entity_id: "verified_originator_entity_id".to_string(),
        verified_beneficiary_entity_id: "verified_beneficiary_entity_id".to_string(),
        originator_pubkey: "originator_pubkey".to_string(),
        beneficiary_pubkey: "beneficiary_pubkey".to_string(),
        sender_gateway_network_id: "sender_gateway_network_id".to_string(),
        recipient_gateway_network_id: "recipient_gateway_network_id".to_string(),
        sender_gateway_owner_id: "sender_gateway_owner_id".to_string(),
        receiver_gateway_owner_id: "receiver_gateway_owner_id".to_string(),
    };
    return transfer_proposal_claims_request;
}

pub fn create_transfer_commence_request(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
) -> TransferCommenceRequest {
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
        client_signature: "client_signature1".to_string(),
    };
    return transfer_commence_request;
}

pub fn create_transfer_proposal_receipt_request(
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
) -> TransferProposalReceiptRequest {
    // TODO: remove hard coded values
    let transfer_proposal_receipt_request = TransferProposalReceiptRequest {
        message_type: "message_type1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        asset_asset_id: "asset_asset_id".to_string(),
        asset_profile_id: "asset_profile_id".to_string(),
        verified_originator_entity_id: "verified_originator_entity_id".to_string(),
        verified_beneficiary_entity_id: "verified_beneficiary_entity_id".to_string(),
        originator_pubkey: "originator_pubkey".to_string(),
        beneficiary_pubkey: "beneficiary_pubkey".to_string(),
        sender_gateway_network_id: "sender_gateway_network_id".to_string(),
        recipient_gateway_network_id: "recipient_gateway_network_id".to_string(),
        sender_gateway_owner_id: "sender_gateway_owner_id".to_string(),
        receiver_gateway_owner_id: "receiver_gateway_owner_id".to_string(),
    };
    return transfer_proposal_receipt_request;
}

pub fn create_ack_commence_request(
    transfer_commence_request: TransferCommenceRequest,
) -> AckCommenceRequest {
    // TODO: remove hard coded values
    let ack_commence_request = AckCommenceRequest {
        message_type: "message_type1".to_string(),
        session_id: "session_id1".to_string(),
        transfer_context_id: "transfer_context_id1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        hash_prev_message: "hash_prev_message1".to_string(),
        server_transfer_number: "server_transfer_number1".to_string(),
        server_signature: "server_signature1".to_string(),
    };
    return ack_commence_request;
}

pub fn create_lock_assertion_request(
    ack_commence_request: AckCommenceRequest,
) -> LockAssertionRequest {
    // TODO: remove hard coded values
    let lock_assertion_request = LockAssertionRequest {
        message_type: "message_type1".to_string(),
        session_id: "session_id1".to_string(),
        transfer_context_id: "transfer_context_id1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        hash_prev_message: "hash_prev_message1".to_string(),
        lock_assertion_claim: "lock_assertion_claim1".to_string(),
        lock_assertion_claim_format: "lock_assertion_claim_format1".to_string(),
        lock_assertion_expiration: "lock_assertion_expiration".to_string(),
        client_transfer_number: "client_transfer_number1".to_string(),
        client_signature: "client_signature1".to_string(),
    };
    return lock_assertion_request;
}

pub fn create_lock_assertion_receipt_request(
    lock_assertion_request: LockAssertionRequest,
) -> LockAssertionReceiptRequest {
    // TODO: remove hard coded values
    let lock_assertion_receipt_request = LockAssertionReceiptRequest {
        message_type: "message_type1".to_string(),
        session_id: "session_id1".to_string(),
        transfer_context_id: "transfer_context_id1".to_string(),
        client_identity_pubkey: "client_identity_pubkey1".to_string(),
        server_identity_pubkey: "server_identity_pubkey1".to_string(),
        hash_prev_message: "hash_prev_message1".to_string(),
        server_transfer_number: "server_transfer_number1".to_string(),
        server_signature: "server_signature1".to_string(),
    };
    return lock_assertion_receipt_request;
}

pub fn get_satp_requests_local_db(conf: Config) -> Database {
    let db = Database {
        db_path: format!(
            "{}{}",
            conf.get_str(SATP_DB_PATH).unwrap(),
            SATP_REQUESTS_DB_PATH
        ),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn get_satp_requests_remote_db(conf: Config) -> Database {
    let db = Database {
        db_path: format!(
            "{}{}",
            conf.get_str(SATP_DB_PATH).unwrap(),
            SATP_REMOTE_REQUESTS_DB_PATH
        ),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn get_satp_requests_states_local_db(conf: Config) -> Database {
    let db = Database {
        db_path: format!(
            "{}{}",
            conf.get_str(SATP_DB_PATH).unwrap(),
            SATP_REQUESTS_STATES_DB_PATH
        ),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn get_satp_requests_states_remote_db(conf: Config) -> Database {
    let db = Database {
        db_path: format!(
            "{}{}",
            conf.get_str(SATP_DB_PATH).unwrap(),
            SATP_REMOTE_REQUESTS_STATES_DB_PATH
        ),
        db_open_max_retries: conf.get_int("db_open_max_retries").unwrap_or(500) as u32,
        db_open_retry_backoff_msec: conf.get_int("db_open_retry_backoff_msec").unwrap_or(10) as u32,
    };
    return db;
}

pub fn log_request_state_in_local_satp_db(
    request_id: &String,
    target: &RequestState,
    conf: Config,
) -> Result<std::option::Option<IVec>, error::Error> {
    let db = get_satp_requests_states_local_db(conf);
    return db.set(&request_id, &target);
}

pub fn log_request_in_local_satp_db<T: Serialize>(
    request_id: &String,
    value: T,
    conf: Config,
) -> Result<std::option::Option<IVec>, error::Error> {
    let db = get_satp_requests_local_db(conf);
    return db.set(&request_id, &value);
}

pub fn log_request_in_remote_satp_db<T: Serialize>(
    request_id: &String,
    value: T,
    conf: Config,
) -> Result<std::option::Option<IVec>, error::Error> {
    let db = get_satp_requests_remote_db(conf);
    return db.set(&request_id, &value);
}

pub fn get_request_from_remote_satp_db<T: DeserializeOwned>(
    request_id: &String,
    conf: Config,
) -> Result<T, error::Error> {
    let db = get_satp_requests_remote_db(conf);
    let query: Result<T, error::Error> = db
        .get::<T>(request_id.to_string())
        .map_err(|e| Error::GetQuery(format!("Failed to get query from db. Error: {:?}", e)));
    return query;
}

pub fn update_request_state_in_local_satp_db(
    request_id: String,
    new_status: request_state::Status,
    state: Option<request_state::State>,
    conf: Config,
) {
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

pub fn get_relay_from_transfer_proposal_claims(
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
) -> (String, String) {
    // TODO
    return ("localhost".to_string(), "9085".to_string());
}

pub fn get_relay_from_transfer_proposal_receipt(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
) -> (String, String) {
    // TODO
    return ("localhost".to_string(), "9085".to_string());
}

pub fn get_relay_from_transfer_commence(
    transfer_commence_request: TransferCommenceRequest,
) -> (String, String) {
    // TODO
    return ("localhost".to_string(), "9085".to_string());
}

pub fn get_relay_from_ack_commence(ack_commence_request: AckCommenceRequest) -> (String, String) {
    // TODO
    return ("localhost".to_string(), "9085".to_string());
}

pub fn get_relay_from_lock_assertion(
    lock_assertion_request: LockAssertionRequest,
) -> (String, String) {
    // TODO
    return ("localhost".to_string(), "9085".to_string());
}

pub fn get_relay_from_lock_assertion_receipt(
    lock_assertion_receipt_request: LockAssertionReceiptRequest,
) -> (String, String) {
    // TODO
    return ("localhost".to_string(), "9085".to_string());
}

pub fn get_relay_params(relay_host: String, relay_port: String, conf: Config) -> (bool, String) {
    let relays_table = conf.get_table("relays").unwrap();
    let mut relay_tls = false;
    let mut relay_tlsca_cert_path = "".to_string();
    for (_relay_name, relay_spec) in relays_table {
        let relay_uri = relay_spec.clone().try_into::<LocationSegment>().unwrap();
        if relay_host == relay_uri.hostname && relay_port == relay_uri.port {
            relay_tls = relay_uri.tls;
            relay_tlsca_cert_path = relay_uri.tlsca_cert_path;
        }
    }
    (relay_tls, relay_tlsca_cert_path)
}

fn create_client_address(relay_host: String, relay_port: String) -> String {
    return format!("http://{}:{}", relay_host, relay_port);
}

pub async fn create_satp_client(
    relay_host: String,
    relay_port: String,
    use_tls: bool,
    tlsca_cert_path: String,
) -> Result<SatpClient<Channel>, Box<dyn std::error::Error>> {
    let client_addr = create_client_address(relay_host.clone(), relay_port.clone());
    let satp_client;
    if use_tls {
        let pem = tokio::fs::read(tlsca_cert_path).await?;
        let ca = Certificate::from_pem(pem);

        let tls = ClientTlsConfig::new()
            .ca_certificate(ca)
            .domain_name(relay_host);

        let channel = Channel::from_shared(client_addr)?
            .tls_config(tls)?
            .connect()
            .await?;

        satp_client = SatpClient::new(channel);
    } else {
        satp_client = SatpClient::connect(client_addr).await?;
    }
    return Ok(satp_client);
}

pub fn get_request_id_from_transfer_proposal_claims(
    request: TransferProposalClaimsRequest,
) -> String {
    // TODO
    return "hard_coded_transfer_proposal_claims_request_id".to_string();
}

pub fn get_request_id_from_transfer_proposal_receipt(
    request: TransferProposalReceiptRequest,
) -> String {
    // TODO
    return "hard_coded_transfer_proposal_receipt_request_id".to_string();
}
