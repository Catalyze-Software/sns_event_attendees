use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use ic_scalable_misc::enums::api_error_type::ApiError;

use shared::attendee_model::{Attendee, InviteAttendeeResponse, JoinedAttendeeResponse};

use super::store::Store;

#[update]
#[candid_method(update)]
async fn join_event(
    event_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {
    Store::join_event(caller(), event_identifier, group_identifier).await
}

#[update]
#[candid_method(update)]
async fn invite_to_event(
    event_identifier: Principal,
    attendee_principal: Principal,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {
    match Store::can_write(caller(), group_identifier, member_identifier).await {
        Ok(_) => Store::invite_to_event(event_identifier, attendee_principal, group_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn accept_user_request_event_invite(
    attendee_principal: Principal,
    event_identifier: Principal,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {
    match Store::can_write(caller(), group_identifier, member_identifier).await {
        Ok(_) => Store::accept_user_request_event_invite(attendee_principal, event_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn accept_owner_request_event_invite(
    event_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {
    Store::accept_owner_request_event_invite(caller(), event_identifier)
}

#[query]
#[candid_method(query)]
fn get_event_attendees_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_event_attendees_count(event_identifiers)
}

#[query]
#[candid_method(query)]
fn get_event_invites_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_group_invites_count(event_identifiers)
}

#[query]
#[candid_method(query)]
fn get_event_attendees(
    event_identifier: Principal,
) -> Result<Vec<JoinedAttendeeResponse>, ApiError> {
    Ok(Store::get_event_attendees(event_identifier))
}

#[query]
#[candid_method(query)]
fn get_self() -> Result<(Principal, Attendee), ApiError> {
    Store::get_self(caller())
}

#[update]
#[candid_method(update)]
fn leave_event(event_identifier: Principal) -> Result<(), ApiError> {
    Store::leave_event(caller(), event_identifier)
}

#[update]
#[candid_method(update)]
fn remove_invite(event_identifier: Principal) -> Result<(), ApiError> {
    Store::remove_invite(caller(), event_identifier)
}

#[update]
#[candid_method(update)]
async fn remove_attendee_from_event(
    attendee_principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<(), ApiError> {
    match Store::can_delete(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::remove_join_from_attendee(attendee_principal, event_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn remove_attendee_invite_from_event(
    principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<(), ApiError> {
    match Store::can_delete(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Store::remove_invite_from_event(principal, event_identifier),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
async fn get_event_invites(
    event_identifier: Principal,
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<Vec<InviteAttendeeResponse>, ApiError> {
    match Store::can_read(caller(), group_identifier, member_identifier).await {
        Ok(_caller) => Ok(Store::get_event_invites(event_identifier)),
        Err(err) => Err(err),
    }
}

#[update]
#[candid_method(update)]
fn add_owner_as_attendee(
    user_principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), bool> {
    Store::add_owner_as_attendee(user_principal, event_identifier, group_identifier)
}
