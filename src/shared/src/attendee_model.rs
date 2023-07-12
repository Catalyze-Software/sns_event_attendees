use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

pub type EventIdentifier = Principal;
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Attendee {
    pub principal: Principal,
    pub joined: HashMap<EventIdentifier, Join>,
    pub invites: HashMap<EventIdentifier, Invite>,
}

impl Default for Attendee {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
            joined: Default::default(),
            invites: Default::default(),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Join {
    pub group_identifier: Principal,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Invite {
    pub group_identifier: Principal,
    pub invite_type: InviteType,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum InviteType {
    None,
    OwnerRequest,
    UserRequest,
}

impl Default for InviteType {
    fn default() -> Self {
        InviteType::None
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct JoinedAttendeeResponse {
    pub event_identifier: Principal,
    pub group_identifier: Principal,
    pub attendee_identifier: Principal,
    pub principal: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct InviteAttendeeResponse {
    pub event_identifier: Principal,
    pub group_identifier: Principal,
    pub attendee_identifier: Principal,
    pub principal: Principal,
    pub invite_type: InviteType,
}
