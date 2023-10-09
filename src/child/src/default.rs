use candid::Principal;
use ic_cdk::{caller, init, post_upgrade, pre_upgrade, query, update};

#[allow(unused_imports)]
use ic_scalable_canister::{
    ic_methods,
    store::{Data, Metadata},
};
use ic_scalable_misc::{
    enums::api_error_type::ApiError,
    models::http_models::{HttpRequest, HttpResponse},
};

use crate::store::{DATA, ENTRIES, STABLE_DATA};

#[update]
pub fn migrate_to_stable() {
    if caller().to_string()
        != "ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe".to_string()
    {
        return;
    }
    let data = DATA.with(|d| d.borrow().clone());
    let _ = STABLE_DATA.with(|s| {
        s.borrow_mut().set(Data {
            name: data.name.clone(),
            identifier: data.identifier.clone(),
            current_entry_id: data.current_entry_id.clone(),
            parent: data.parent.clone(),
            is_available: data.is_available.clone(),
            updated_at: data.updated_at.clone(),
            created_at: data.created_at.clone(),
        })
    });

    let _ = ENTRIES.with(|e| {
        data.entries.iter().for_each(|entry| {
            e.borrow_mut().insert(entry.0.to_string(), entry.1.clone());
        });
    });
}

#[query]
pub fn sanity_check() -> String {
    STABLE_DATA.with(|data| Data::get_name(data.borrow().get()))
}

#[query]
pub fn get_metadata() -> Result<Metadata, ApiError> {
    STABLE_DATA.with(|data| ENTRIES.with(|entries| Data::get_metadata(data, entries)))
}

#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| ic_methods::deprecated_pre_upgrade(data))
}

#[post_upgrade]
pub fn post_upgrade() {
    DATA.with(|data| ic_methods::deprecated_post_upgrade(data))
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
        ENTRIES.with(|entries| Data::http_request_with_metrics(data, entries, req, vec![]))
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

    use ic_cdk::api::management_canister::http_request::HttpResponse;
    use ic_scalable_canister::store::Metadata;
    use ic_scalable_misc::enums::api_error_type::ApiError;
    use ic_scalable_misc::models::http_models::HttpRequest;
    export_service!();
    __export_service()
}

// Method used to save the candid interface to a file
#[test]
pub fn candid() {
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    save_candid(__export_did_tmp_(), String::from("child"));
}
