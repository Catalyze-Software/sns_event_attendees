use candid::Principal;
use ic_cdk::{
    api::{
        call::RejectionCode,
        management_canister::{
            main::{canister_status as _canister_status, CanisterStatusResponse},
            provisional::CanisterIdRecord,
        },
    },
    caller, id, init, query, update,
};

use ic_scalable_canister::ic_scalable_misc::{
    enums::api_error_type::ApiError,
    models::http_models::{HttpRequest, HttpResponse},
};
#[allow(unused_imports)]
use ic_scalable_canister::{
    ic_methods,
    store::{Data, Metadata},
};

use crate::store::{ENTRIES, STABLE_DATA};

#[query]
pub fn sanity_check() -> String {
    STABLE_DATA.with(|data| Data::get_name(data.borrow().get()))
}

#[update]
async fn add_entry_by_parent(entry: Vec<u8>) -> Result<(), ApiError> {
    STABLE_DATA.with(|v| {
        ENTRIES.with(|entries| {
            Data::add_entry_by_parent(v, entries, caller(), entry, Some("eae".to_string()))
        })
    })
}

#[update]
fn accept_cycles() -> u64 {
    ic_methods::accept_cycles()
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    STABLE_DATA.with(|data| {
        ENTRIES.with(|entries| {
            Data::http_request_with_metrics(data, entries.borrow().len() as usize, req, vec![])
        })
    })
}

#[init]
pub fn init(parent: Principal, name: String, identifier: usize) {
    STABLE_DATA.with(|data| {
        ic_methods::init(data, parent, name, identifier);
    })
}

// Hacky way to expose the candid interface to the outside world
#[query(name = "__get_candid_interface_tmp_hack")]
pub fn __export_did_tmp_() -> String {
    use candid::export_service;
    use candid::Principal;
    use shared::attendee_model::*;

    use ic_canister_backup::models::*;
    use ic_cdk::api::management_canister::http_request::HttpResponse;
    use ic_scalable_canister::ic_scalable_misc::enums::api_error_type::ApiError;
    use ic_scalable_canister::ic_scalable_misc::models::http_models::HttpRequest;
    export_service!();
    __export_service()
}

// Method used to save the candid interface to a file
#[test]
pub fn candid() {
    use ic_scalable_canister::ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("child"));
}

#[update(guard = "is_monitor")]
async fn canister_status() -> Result<(CanisterStatusResponse,), (RejectionCode, String)> {
    _canister_status(CanisterIdRecord { canister_id: id() }).await
}

pub fn is_monitor() -> Result<(), String> {
    const OWNERS: [&str; 1] = ["6or45-oyaaa-aaaap-absua-cai"];

    match OWNERS.iter().any(|p| p == &caller().to_string()) {
        true => Ok(()),
        false => Err("Unauthorized".to_string()),
    }
}
