use std::{collections::HashMap, iter::FromIterator};

use candid::{candid_method, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use ic_scalable_misc::enums::api_error_type::ApiError;

use shared::attendee_model::{Attendee, InviteAttendeeResponse, JoinedAttendeeResponse};

use crate::store::DATA;

use super::store::Store;

#[update]
#[candid_method(update)]
pub fn migration_add_event_attendees(attendees: Vec<(Principal, Attendee)>) -> () {
    DATA.with(|data| {
        data.borrow_mut().entries = HashMap::from_iter(attendees);
    })
}

// Method to join an existing event
// The method is async because it optionally creates a new canister is created
#[update]
#[candid_method(update)]
async fn join_event(
    event_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {
    Store::join_event(caller(), event_identifier, group_identifier).await
}

// Method to invite a member to an event
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

// Method to accept an invite to an event as a admin
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

// Method to accept an invite to an event as a user
#[update]
#[candid_method(update)]
async fn accept_owner_request_event_invite(
    event_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {
    Store::accept_owner_request_event_invite(caller(), event_identifier)
}

// Method to get the number of attendees for an event
#[query]
#[candid_method(query)]
fn get_event_attendees_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_event_attendees_count(event_identifiers)
}

// Method to get the number of invites for an event
#[query]
#[candid_method(query)]
fn get_event_invites_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
    Store::get_group_invites_count(event_identifiers)
}

// Method to get the attendees for an event
#[query]
#[candid_method(query)]
fn get_event_attendees(
    event_identifier: Principal,
) -> Result<Vec<JoinedAttendeeResponse>, ApiError> {
    Ok(Store::get_event_attendees(event_identifier))
}

// Method to get the caller his joined events and invites
#[query]
#[candid_method(query)]
fn get_self() -> Result<(Principal, Attendee), ApiError> {
    Store::get_self(caller())
}

// Method to leave an event as a user
#[update]
#[candid_method(update)]
fn leave_event(event_identifier: Principal) -> Result<(), ApiError> {
    Store::remove_join_from_attendee(caller(), event_identifier)
}

// Method to remove an event invite as a user
#[update]
#[candid_method(update)]
fn remove_invite(event_identifier: Principal) -> Result<(), ApiError> {
    Store::remove_invite_from_event(caller(), event_identifier)
}

// Method to remove an event attendee as a admin
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

// Method to remove an event invite as a admin
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

// Method to get event invites for a specific event inside a group
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

// Method to add the owner as an attendee
#[update]
#[candid_method(update)]
fn add_owner_as_attendee(
    user_principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), bool> {
    Store::add_owner_as_attendee(user_principal, event_identifier, group_identifier)
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
#[candid_method(query)]
fn get_chunked_join_data(
    event_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_join_data(&event_identifier, chunk, max_bytes_per_chunk)
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
#[candid_method(query)]
fn get_chunked_invite_data(
    event_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_invite_data(&event_identifier, chunk, max_bytes_per_chunk)
}
