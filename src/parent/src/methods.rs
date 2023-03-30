use candid::{candid_method, Principal};
use ic_cdk::query;
use ic_scalable_misc::models::paged_response_models::PagedResponse;

use shared::attendee_model::{InviteAttendeeResponse, JoinedAttendeeResponse};

use super::store::ScalableData;

// Method used to get all the members from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
#[query(composite = true)]
#[candid_method(query)]
async fn get_members(
    group_identifier: Principal,
    limit: usize,
    page: usize,
) -> PagedResponse<JoinedAttendeeResponse> {
    ScalableData::get_joined_child_canister_data(group_identifier, limit, page).await
}

// Method used to get all the members from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
#[query(composite = true)]
#[candid_method(query)]
async fn get_invites(
    group_identifier: Principal,
    limit: usize,
    page: usize,
) -> PagedResponse<InviteAttendeeResponse> {
    ScalableData::get_invites_child_canister_data(group_identifier, limit, page).await
}
