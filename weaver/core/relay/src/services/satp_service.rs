// Internal generated modules
use weaverpb::common::ack::{ack, Ack};
use weaverpb::relay::satp::satp_server::Satp;
use weaverpb::relay::satp::{
    AckCommenceRequest, AckFinalReceiptRequest, CommitFinalAssertionRequest, CommitPrepareRequest,
    CommitReadyRequest, LockAssertionReceiptRequest, LockAssertionRequest, SendAssetStatusRequest,
    TransferCommenceRequest, TransferCompletedRequest, TransferProposalClaimsRequest,
    TransferProposalReceiptRequest,
};

// Internal modules
use crate::error::Error;
use crate::relay_proto::parse_address;
use crate::services::satp_helper::{
    create_ack_error_message, get_request_id_from_transfer_proposal_receipt,
    log_request_in_local_satp_db, log_request_in_remote_satp_db,
};

use super::helpers::get_driver;
// external modules
use super::satp_helper::{
    create_ack_commence_request, create_ack_final_receipt_request,
    create_commit_final_assertion_request, create_commit_prepare_request,
    create_commit_ready_request, create_lock_assertion_receipt_request,
    create_lock_assertion_request, create_transfer_commence_request,
    create_transfer_completed_request, create_transfer_proposal_receipt_request,
    get_driver_address_from_ack_commence, get_relay_from_ack_commence,
    get_relay_from_ack_final_receipt, get_relay_from_commit_final_assertion,
    get_relay_from_commit_prepare, get_relay_from_commit_ready, get_relay_from_lock_assertion,
    get_relay_from_lock_assertion_receipt, get_relay_from_send_asset_status,
    get_relay_from_transfer_commence, get_relay_from_transfer_proposal_claims,
    get_relay_from_transfer_proposal_receipt, get_relay_params,
    get_request_id_from_transfer_proposal_claims, spawn_send_ack_commence_request,
    spawn_send_ack_final_receipt_broadcast_request, spawn_send_assign_asset_request,
    spawn_send_commit_prepare_request, spawn_send_create_asset_request,
    spawn_send_extinguish_request, spawn_send_lock_assertion_broadcast_request,
    spawn_send_lock_assertion_request, spawn_send_perform_lock_request,
    spawn_send_transfer_commence_request, spawn_send_transfer_proposal_receipt_request,
};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct SatpService {
    pub config_lock: RwLock<config::Config>,
}

/// AssetTransferService is the gRPC server implementation that handles the logic for
/// communication of the asset transfer protocol SATP between two gateways.
#[tonic::async_trait]
impl Satp for SatpService {
    /// transfer_proposal_claims is run on the receiver gateway to allow the sender gateway to initiate an asset transfer.
    async fn transfer_proposal_claims(
        &self,
        request: Request<TransferProposalClaimsRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got a TransferProposalClaimsRequest from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let transfer_proposal_claims_request = request.into_inner().clone();
        let request_id =
            get_request_id_from_transfer_proposal_claims(transfer_proposal_claims_request.clone());
        let conf = self.config_lock.read().await;

        match log_request_in_remote_satp_db(
            &request_id,
            &transfer_proposal_claims_request,
            conf.clone(),
        ) {
            Ok(_) => {
                println!("Successfully stored TransferProposalClaimsRequest in remote satp_db with request_id: {}", request_id);
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing TransferProposalClaimsRequest in remote satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_transfer_proposal_claims_request(
            transfer_proposal_claims_request,
            conf.clone(),
        ) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of transfer proposal claims request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Transfer proposal claims failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn transfer_proposal_receipt(
        &self,
        request: Request<TransferProposalReceiptRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got an ack transfer proposal receipt request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let transfer_proposal_receipt_request = request.into_inner().clone();
        let request_id = get_request_id_from_transfer_proposal_receipt(
            transfer_proposal_receipt_request.clone(),
        );
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> = log_request_in_local_satp_db(
            &request_id,
            &transfer_proposal_receipt_request,
            conf.clone(),
        );
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored TransferProposalReceiptRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing TransferProposalReceiptRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_transfer_proposal_receipt_request(
            transfer_proposal_receipt_request,
            conf.clone(),
        ) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of transfer proposal receipt request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Ack transfer proposal receipt failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    /// transfer_commence is run on the receiver gateway to allow the sender gateway to signal to the
    /// receiver gateway that it is ready to start the transfer of the digital asset
    async fn transfer_commence(
        &self,
        request: Request<TransferCommenceRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got a TransferCommenceRequest from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let transfer_commence_request = request.into_inner().clone();
        let request_id = transfer_commence_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        match log_request_in_remote_satp_db(&request_id, &transfer_commence_request, conf.clone()) {
            Ok(_) => {
                println!("Successfully stored TransferCommenceRequest in remote satp_db with request_id: {}", request_id);
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing TransferCommenceRequest in remote satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_transfer_commence_request(transfer_commence_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of transfer commence request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Transfer commence failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn ack_commence(
        &self,
        request: Request<AckCommenceRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got an ack commence request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let ack_commence_request = request.into_inner().clone();
        let request_id = ack_commence_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> =
            log_request_in_local_satp_db(&request_id, &ack_commence_request, conf.clone());
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored AckCommenceRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing AckCommenceRequest in local satp_db for request_id".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_ack_commence_request(ack_commence_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!("Sending Ack of ack commence request back: {:?}\n", reply);
                reply
            }
            Err(e) => {
                let error_message = "Ack commence failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn send_asset_status(
        &self,
        request: Request<SendAssetStatusRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got a send asset status request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let send_asset_status_request = request.into_inner().clone();
        let request_id = send_asset_status_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> =
            log_request_in_local_satp_db(&request_id, &send_asset_status_request, conf.clone());
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored SendAssetStatusRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing SendAssetStatusRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_send_asset_status_request(send_asset_status_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of send asset status request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Send asset status failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn lock_assertion(
        &self,
        request: Request<LockAssertionRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got a LockAssertionRequest from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let lock_assertion_request = request.into_inner().clone();
        let request_id = lock_assertion_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        match log_request_in_remote_satp_db(&request_id, &lock_assertion_request, conf.clone()) {
            Ok(_) => {
                println!("Successfully stored LockAssertionRequest in remote satp_db with request_id: {}", request_id);
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing LockAssertionRequest in remote satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_lock_assertion_request(lock_assertion_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!("Sending Ack of lock assertion request back: {:?}\n", reply);
                reply
            }
            Err(e) => {
                let error_message = "Lock assertion failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn lock_assertion_receipt(
        &self,
        request: Request<LockAssertionReceiptRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got an lock assertion receipt request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let lock_assertion_receipt_request = request.into_inner().clone();
        let request_id = lock_assertion_receipt_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> = log_request_in_local_satp_db(
            &request_id,
            &lock_assertion_receipt_request,
            conf.clone(),
        );
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored LockAssertionReceiptRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing LockAssertionReceiptRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_lock_assertion_receipt_request(lock_assertion_receipt_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of lock assertion receipt request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Lock assertion receipt failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn commit_prepare(
        &self,
        request: Request<CommitPrepareRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got commit prepare request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let commit_prepare_request = request.into_inner().clone();
        let request_id = commit_prepare_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> =
            log_request_in_local_satp_db(&request_id, &commit_prepare_request, conf.clone());
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored CommitPrepareRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing CommitPrepareRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_commit_prepare_request(commit_prepare_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!("Sending Ack of commit prepare request back: {:?}\n", reply);
                reply
            }
            Err(e) => {
                let error_message = "Commit prepare failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn commit_ready(
        &self,
        request: Request<CommitReadyRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got commit ready request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let commit_ready_request = request.into_inner().clone();
        let request_id = commit_ready_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> =
            log_request_in_local_satp_db(&request_id, &commit_ready_request, conf.clone());
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored CommitReadyRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing CommitReadyRequest in local satp_db for request_id".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_commit_ready_request(commit_ready_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!("Sending Ack of commit ready request back: {:?}\n", reply);
                reply
            }
            Err(e) => {
                let error_message = "Commit ready failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn commit_final_assertion(
        &self,
        request: Request<CommitFinalAssertionRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got commit final assertion request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let commit_final_assertion_request = request.into_inner().clone();
        let request_id = commit_final_assertion_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> = log_request_in_local_satp_db(
            &request_id,
            &commit_final_assertion_request,
            conf.clone(),
        );
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored CommitFinalAssertionRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing CommitFinalAssertionRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_commit_final_assertion_request(commit_final_assertion_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of commit final assertion request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Commit final assertion failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn ack_final_receipt(
        &self,
        request: Request<AckFinalReceiptRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got commit final assertion request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let ack_final_receipt_request = request.into_inner().clone();
        let request_id = ack_final_receipt_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> =
            log_request_in_local_satp_db(&request_id, &ack_final_receipt_request, conf.clone());
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored AckFinalReceiptRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing AckFinalReceiptRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        match process_ack_final_receipt_request(ack_final_receipt_request, conf.clone()) {
            Ok(ack) => {
                let reply = Ok(Response::new(ack));
                println!(
                    "Sending Ack of ack final receipt request back: {:?}\n",
                    reply
                );
                reply
            }
            Err(e) => {
                let error_message = "Ack final receipt failed.".to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                reply
            }
        }
    }

    async fn transfer_completed(
        &self,
        request: Request<TransferCompletedRequest>,
    ) -> Result<Response<Ack>, Status> {
        println!(
            "Got commit final assertion request from {:?} - {:?}",
            request.remote_addr(),
            request
        );

        let transfer_completed_request = request.into_inner().clone();
        let request_id = transfer_completed_request.session_id.to_string();
        let conf = self.config_lock.read().await;

        // TODO refactor
        let request_logged: Result<Option<sled::IVec>, Error> =
            log_request_in_local_satp_db(&request_id, &transfer_completed_request, conf.clone());
        match request_logged {
            Ok(_) => {
                println!(
                    "Successfully stored TransferCompletedRequest in local satp_db with request_id: {}",
                    request_id
                )
            }
            Err(e) => {
                // Internal failure of sled. Send Error response
                let error_message =
                    "Error storing TransferCompletedRequest in local satp_db for request_id"
                        .to_string();
                let reply = create_ack_error_message(request_id, error_message, e);
                return reply;
            }
        }

        let reply = Ack {
            status: ack::Status::Ok as i32,
            request_id: request_id.to_string(),
            message: "Ack of the Transfer Completed request".to_string(),
        };
        return Ok(Response::new(reply));
    }
}

pub fn process_transfer_proposal_claims_request(
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id =
        get_request_id_from_transfer_proposal_claims(transfer_proposal_claims_request.clone());
    let is_valid_request =
        is_valid_transfer_proposal_claims_request(transfer_proposal_claims_request.clone());

    if is_valid_request {
        println!("The transfer proposal claims request is valid\n");
        match send_transfer_proposal_receipt_request(transfer_proposal_claims_request, conf) {
            Ok(ack) => {
                println!("Ack transfer proposal claims request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: Ack transfer proposal claims failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The transfer proposal claims request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The transfer proposal claims request is invalid".to_string(),
        });
    }
}

pub fn process_transfer_proposal_receipt_request(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id =
        get_request_id_from_transfer_proposal_receipt(transfer_proposal_receipt_request.clone());
    let is_valid_request =
        is_valid_transfer_proposal_receipt_request(transfer_proposal_receipt_request.clone());

    if is_valid_request {
        println!("The transfer proposal receipt request is valid\n");
        match send_transfer_commence_request(transfer_proposal_receipt_request, conf) {
            Ok(ack) => {
                println!("Ack transfer proposal receipt request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: Ack transfer proposal receipt failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The transfer proposal receipt request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The transfer proposal receipt request is invalid".to_string(),
        });
    }
}

/// process_transfer_commence_request is run on the receiver gateway to initiate asset transfer protocol that was
/// requested from the sender gateway
pub fn process_transfer_commence_request(
    transfer_commence_request: TransferCommenceRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = transfer_commence_request.session_id.to_string();
    let is_valid_request = is_valid_transfer_commence_request(transfer_commence_request.clone());

    if is_valid_request {
        println!("The transfer commence request is valid\n");
        match send_ack_commence_request(transfer_commence_request, conf) {
            Ok(ack) => {
                println!("Ack transfer commence request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: Ack commence request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The transfer commence request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The transfer commence request is invalid".to_string(),
        });
    }
}

pub fn process_tranfer_proposal_receipt_request(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id =
        get_request_id_from_transfer_proposal_receipt(transfer_proposal_receipt_request.clone());
    let is_valid_request =
        is_valid_transfer_proposal_receipt_request(transfer_proposal_receipt_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The transfer proposal receipt request is valid\n");
        match send_transfer_commence_request(transfer_proposal_receipt_request, conf) {
            Ok(ack) => {
                println!("Ack transfer proposal receipt request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: Ack transfer proposal receipt failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The transfer proposal receipt request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The transfer proposal receipt request is invalid".to_string(),
        });
    }
}

/// process_ack_commence_request is invoked by the receiver gateway to ack the transfer commence request
/// requested ed by the sender gateway
pub fn process_ack_commence_request(
    ack_commence_request: AckCommenceRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = ack_commence_request.session_id.to_string();
    let is_valid_request = is_valid_ack_commence_request(ack_commence_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The ack commence request is valid\n");
        match send_perform_lock_request(ack_commence_request, conf) {
            Ok(ack) => {
                println!("Ack ack commence request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: perform lock request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The ack commence request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The ack commence request is invalid".to_string(),
        });
    }
}

pub fn process_send_asset_status_request(
    send_asset_status_request: SendAssetStatusRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = send_asset_status_request.session_id.to_string();
    let is_valid_request = is_valid_send_asset_status_request(send_asset_status_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The send asset status request is valid\n");
        match send_lock_assertion_request(send_asset_status_request, conf) {
            Ok(ack) => {
                println!("Ack send asset status request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: send asset status request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The send asset status request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The send asset status request is invalid".to_string(),
        });
    }
}

pub fn process_lock_assertion_request(
    lock_assertion_request: LockAssertionRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = lock_assertion_request.session_id.to_string();
    let is_valid_request = is_valid_lock_assertion_request(lock_assertion_request.clone());

    if is_valid_request {
        println!("The lock assertion request is valid\n");
        match send_lock_assertion_receipt_request(lock_assertion_request, conf) {
            Ok(ack) => {
                println!("Ack lock assertion request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: Ack lock assertion failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The lock assertion request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The lock assertion request is invalid".to_string(),
        });
    }
}

pub fn process_lock_assertion_receipt_request(
    lock_assertion_receipt_request: LockAssertionReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = lock_assertion_receipt_request.session_id.to_string();
    let is_valid_request =
        is_valid_lock_assertion_receipt_request(lock_assertion_receipt_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The lock assertion receipt request is valid\n");
        match send_commit_prepare_request(lock_assertion_receipt_request, conf) {
            Ok(ack) => {
                println!("Ack lock assertion receipt request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: perform lock request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The lock assertion receipt request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The lock assertion receipt request is invalid".to_string(),
        });
    }
}

pub fn process_commit_prepare_request(
    commit_prepare_request: CommitPrepareRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = commit_prepare_request.session_id.to_string();
    let is_valid_request = is_valid_commit_prepare_request(commit_prepare_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The commit prepare request is valid\n");
        match send_commit_ready_request(commit_prepare_request, conf) {
            Ok(ack) => {
                println!("Ack commit prepare request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: commit prepare request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The commit prepare request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The commit prepare request is invalid".to_string(),
        });
    }
}

pub fn process_commit_ready_request(
    commit_ready_request: CommitReadyRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = commit_ready_request.session_id.to_string();
    let is_valid_request = is_valid_commit_ready_request(commit_ready_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The commit ready request is valid\n");
        match send_commit_final_assertion_request(commit_ready_request, conf) {
            Ok(ack) => {
                println!("Ack commit ready request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: commit ready request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The commit ready request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The commit ready request is invalid".to_string(),
        });
    }
}

pub fn process_commit_final_assertion_request(
    commit_final_assertion_request: CommitFinalAssertionRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = commit_final_assertion_request.session_id.to_string();
    let is_valid_request =
        is_valid_commit_final_assertion_request(commit_final_assertion_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The commit final assertion request is valid\n");
        match send_ack_final_receipt_request(commit_final_assertion_request, conf) {
            Ok(ack) => {
                println!("Ack commit final assertion request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: commit final assertion request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The commit final assertion request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The commit final assertion request is invalid".to_string(),
        });
    }
}

pub fn process_ack_final_receipt_request(
    ack_final_receipt_request: AckFinalReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = ack_final_receipt_request.session_id.to_string();
    let is_valid_request = is_valid_ack_final_receipt_request(ack_final_receipt_request.clone());

    // TODO some processing
    if is_valid_request {
        println!("The ack final receipt request is valid\n");
        match send_transfer_completed_request(ack_final_receipt_request, conf) {
            Ok(ack) => {
                println!("Ack ack final receipt request.");
                let reply = Ok(ack);
                println!("Sending back Ack: {:?}\n", reply);
                reply
            }
            Err(e) => {
                return Ok(Ack {
                    status: ack::Status::Error as i32,
                    request_id: request_id.to_string(),
                    message: format!("Error: ack final receipt request failed. {:?}", e),
                });
            }
        }
    } else {
        println!("The ack final receipt request is invalid\n");
        return Ok(Ack {
            status: ack::Status::Error as i32,
            request_id: request_id.to_string(),
            message: "Error: The ack final receipt request is invalid".to_string(),
        });
    }
}

fn send_transfer_proposal_receipt_request(
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id =
        get_request_id_from_transfer_proposal_claims(transfer_proposal_claims_request.clone());
    let (relay_host, relay_port) =
        get_relay_from_transfer_proposal_claims(transfer_proposal_claims_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let transfer_proposal_receipt_request =
        create_transfer_proposal_receipt_request(transfer_proposal_claims_request.clone());

    spawn_send_transfer_proposal_receipt_request(
        transfer_proposal_receipt_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );
    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the Transfer Proposal Claims request".to_string(),
    };
    return Ok(reply);
}

fn send_transfer_commence_request(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id =
        get_request_id_from_transfer_proposal_receipt(transfer_proposal_receipt_request.clone());
    let (relay_host, relay_port) =
        get_relay_from_transfer_proposal_receipt(transfer_proposal_receipt_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let transfer_commence_request =
        create_transfer_commence_request(transfer_proposal_receipt_request.clone());

    spawn_send_transfer_commence_request(
        transfer_commence_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );
    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the Transfer Proposal Claims request".to_string(),
    };
    return Ok(reply);
}

fn send_ack_commence_request(
    transfer_commence_request: TransferCommenceRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = &transfer_commence_request.session_id.to_string();
    let (relay_host, relay_port) =
        get_relay_from_transfer_commence(transfer_commence_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let ack_commence_request = create_ack_commence_request(transfer_commence_request.clone());

    spawn_send_ack_commence_request(
        ack_commence_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );
    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the Commence Response request".to_string(),
    };
    return Ok(reply);
}

fn send_lock_assertion_request(
    send_asset_status_request: SendAssetStatusRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = &send_asset_status_request.session_id.to_string();
    let (relay_host, relay_port) =
        get_relay_from_send_asset_status(send_asset_status_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let lock_assertion_request = create_lock_assertion_request(send_asset_status_request.clone());

    spawn_send_lock_assertion_request(
        lock_assertion_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );
    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the Send Asset Status request".to_string(),
    };
    return Ok(reply);
}

fn send_lock_assertion_receipt_request(
    lock_assertion_request: LockAssertionRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = &lock_assertion_request.session_id.to_string();
    let (relay_host, relay_port) = get_relay_from_lock_assertion(lock_assertion_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let lock_assertion_receipt_request =
        create_lock_assertion_receipt_request(lock_assertion_request.clone());

    spawn_send_lock_assertion_broadcast_request(
        lock_assertion_receipt_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );
    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the Lock Assertion request".to_string(),
    };
    return Ok(reply);
}

fn send_perform_lock_request(
    ack_commence_request: AckCommenceRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = &ack_commence_request.session_id.to_string();
    let driver_address = get_driver_address_from_ack_commence(ack_commence_request.clone());
    let parsed_address = parse_address(driver_address)?;
    let result = get_driver(parsed_address.network_id.to_string(), conf.clone());
    match result {
        Ok(driver_info) => {
            spawn_send_perform_lock_request(driver_info, ack_commence_request);
            let reply = Ack {
                status: ack::Status::Ok as i32,
                request_id: request_id.to_string(),
                message: "Ack of the ack commence request".to_string(),
            };
            return Ok(reply);
        }
        Err(e) => {
            return Ok(Ack {
                status: ack::Status::Error as i32,
                request_id: request_id.to_string(),
                message: format!(
                    "Error: Ack of the ack commence request failed. Driver not found {:?}",
                    e
                ),
            });
        }
    }
}

fn send_commit_prepare_request(
    lock_assertion_receipt_request: LockAssertionReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    let request_id = &lock_assertion_receipt_request.session_id.to_string();
    let (relay_host, relay_port) =
        get_relay_from_lock_assertion_receipt(lock_assertion_receipt_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let commit_prepare_request =
        create_commit_prepare_request(lock_assertion_receipt_request.clone());

    spawn_send_commit_prepare_request(
        commit_prepare_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );
    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the Lock Assertion request".to_string(),
    };
    return Ok(reply);
}

fn send_commit_ready_request(
    commit_prepare_request: CommitPrepareRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    // TODO
    let request_id = &commit_prepare_request.session_id.to_string();
    let (relay_host, relay_port) = get_relay_from_commit_prepare(commit_prepare_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let commit_ready_request = create_commit_ready_request(commit_prepare_request.clone());

    spawn_send_create_asset_request(
        commit_ready_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );

    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the commit prepare request".to_string(),
    };
    return Ok(reply);
}

fn send_commit_final_assertion_request(
    commit_ready_request: CommitReadyRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    // TODO
    let request_id = &commit_ready_request.session_id.to_string();
    let (relay_host, relay_port) = get_relay_from_commit_ready(commit_ready_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let commit_final_assertion_request =
        create_commit_final_assertion_request(commit_ready_request.clone());

    spawn_send_extinguish_request(
        commit_final_assertion_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );

    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the commit prepare request".to_string(),
    };
    return Ok(reply);
}

fn send_ack_final_receipt_request(
    commit_final_assertion_request: CommitFinalAssertionRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    // TODO
    let request_id = &commit_final_assertion_request.session_id.to_string();
    let (relay_host, relay_port) =
        get_relay_from_commit_final_assertion(commit_final_assertion_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let ack_final_receipt_request =
        create_ack_final_receipt_request(commit_final_assertion_request.clone());

    spawn_send_assign_asset_request(
        ack_final_receipt_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );

    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the commit prepare request".to_string(),
    };
    return Ok(reply);
}

fn send_transfer_completed_request(
    ack_final_receipt_request: AckFinalReceiptRequest,
    conf: config::Config,
) -> Result<Ack, Error> {
    // TODO
    let request_id = &ack_final_receipt_request.session_id.to_string();
    let (relay_host, relay_port) =
        get_relay_from_ack_final_receipt(ack_final_receipt_request.clone());
    let (use_tls, tlsca_cert_path) =
        get_relay_params(relay_host.clone(), relay_port.clone(), conf.clone());
    let transfer_completed_request =
        create_transfer_completed_request(ack_final_receipt_request.clone());

    spawn_send_ack_final_receipt_broadcast_request(
        transfer_completed_request,
        relay_host,
        relay_port,
        use_tls,
        tlsca_cert_path,
        conf,
    );

    let reply = Ack {
        status: ack::Status::Ok as i32,
        request_id: request_id.to_string(),
        message: "Ack of the commit prepare request".to_string(),
    };
    return Ok(reply);
}

fn is_valid_transfer_proposal_claims_request(
    transfer_proposal_claims_request: TransferProposalClaimsRequest,
) -> bool {
    //TODO
    true
}

fn is_valid_transfer_proposal_receipt_request(
    transfer_proposal_receipt_request: TransferProposalReceiptRequest,
) -> bool {
    //TODO
    true
}

fn is_valid_transfer_commence_request(transfer_commence_request: TransferCommenceRequest) -> bool {
    //TODO
    true
}

fn is_valid_ack_commence_request(ack_commence_request: AckCommenceRequest) -> bool {
    //TODO
    true
}

fn is_valid_lock_assertion_request(lock_assertion_request: LockAssertionRequest) -> bool {
    //TODO
    true
}

fn is_valid_lock_assertion_receipt_request(
    lock_assertion_receipt_request: LockAssertionReceiptRequest,
) -> bool {
    //TODO
    true
}

fn is_valid_commit_prepare_request(commit_prepare_request: CommitPrepareRequest) -> bool {
    //TODO
    true
}

fn is_valid_commit_ready_request(commit_ready_request: CommitReadyRequest) -> bool {
    //TODO
    true
}

fn is_valid_commit_final_assertion_request(
    commit_final_assertion_request: CommitFinalAssertionRequest,
) -> bool {
    //TODO
    true
}

fn is_valid_ack_final_receipt_request(ack_final_receipt_request: AckFinalReceiptRequest) -> bool {
    //TODO
    true
}

fn is_valid_transfer_completed_request(
    transfer_completed_request: TransferCompletedRequest,
) -> bool {
    //TODO
    true
}

fn is_valid_send_asset_status_request(send_asset_status_request: SendAssetStatusRequest) -> bool {
    //TODO
    true
}