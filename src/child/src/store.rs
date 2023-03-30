use std::{cell::RefCell, vec};

use candid::Principal;
use ic_cdk::{
    api::{call, time},
    caller, id,
};
use ic_scalable_canister::store::Data;
use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        privacy_type::Privacy,
    },
    helpers::{
        error_helper::api_error,
        role_helper::{default_roles, get_group_roles, get_member_roles, has_permission},
        serialize_helper::serialize,
    },
    models::{
        identifier_model::Identifier,
        permissions_models::{PermissionActionType, PermissionType},
    },
};

use shared::attendee_model::{
    Attendee, Invite, InviteAttendeeResponse, InviteType, Join, JoinedAttendeeResponse,
};

thread_local! {
    pub static DATA: RefCell<Data<Attendee>>  = RefCell::new(Data::default());
}

pub struct Store;

impl Store {
    // TODO: See if i can refactor this code - rem.codes
    // Method to join an existing event
    pub async fn join_event(
        caller: Principal,
        event_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        // Get the group owner and privacy from an inter-canister call
        let event_owner_and_privacy =
            Self::_get_event_privacy_and_owner(event_identifier.clone(), group_identifier.clone())
                .await;

        match event_owner_and_privacy {
            // if the call fails return an error
            Err(err) => Err(err),
            // if the call succeeds, continue
            Ok((_, _event_privacy)) => {
                // get the attendee for the caller
                match Self::_get_attendee_from_caller(caller) {
                    // if the attendee is not found, do nothing
                    None => {}
                    // if the attendee is found, continue
                    Some((_identifier, mut _exisiting_attendee)) => {
                        // if the event identifier is already found in the joined array, throw an error
                        if _exisiting_attendee
                            .joined
                            .iter()
                            .any(|m| &m.event_identifier == &event_identifier)
                        {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "ALREADY_JOINED",
                                "You are already part of this event",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "join_group",
                                None,
                            ));
                        }
                        // if the event identifier is already found in the invites array, throw an error
                        if _exisiting_attendee
                            .invites
                            .iter()
                            .any(|m| &m.event_identifier == &event_identifier)
                        {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "PENDING_INVITE",
                                "There is already a pending invite for this event",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "join_event",
                                None,
                            ));
                        }
                    }
                };

                // add the event invite or join to the attendee
                let updated_attendee = Self::add_invite_or_join_event_to_attendee(
                    caller,
                    event_identifier.clone(),
                    group_identifier,
                    Self::_get_attendee_from_caller(caller),
                    _event_privacy,
                );

                match updated_attendee {
                    // If something went wrong, return the error
                    Err(err) => Err(err),
                    // If the attendee was updated or added, continue
                    Ok(_updated_attendee) => match Self::_get_attendee_from_caller(caller) {
                        None => {
                            let result = DATA.with(|data| {
                                Data::add_entry(data, _updated_attendee, Some("eae".to_string()))
                            });
                            Self::update_attendee_count_on_event(&event_identifier);
                            result
                        }
                        Some((_identifier, _)) => {
                            let result = DATA.with(|data| {
                                Data::update_entry(data, _identifier, _updated_attendee)
                            });
                            Self::update_attendee_count_on_event(&event_identifier);
                            result
                        }
                    },
                }
                // TODO: add scaling logic
                // Determine if an entry needs to be updated or added as a new one
            }
        }
    }

    // Method to remove a event attendee entry from an attendee
    pub fn remove_join_from_attendee(
        attendee_principal: Principal,
        event_identifier: Principal,
    ) -> Result<(), ApiError> {
        match Self::_get_attendee_from_caller(attendee_principal) {
            // if the attendee is not found, return an error
            None => Err(Self::_attendee_not_found_error(
                "remove_join_from_attendee",
                None,
            )),
            // if the attendee is found, continue
            Some((_identifier, mut _attendee)) => {
                // filter out the event from the joined array
                let joined: Vec<Join> = _attendee
                    .joined
                    .into_iter()
                    .filter(|j| &j.event_identifier != &event_identifier)
                    .collect();

                // set the joined array to the filtered array on the attendee
                _attendee.joined = joined;
                // update the attendee
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));

                // update the attendee count on the event canister (fire-and-forget)
                return Ok(Self::update_attendee_count_on_event(&event_identifier));
            }
        }
    }

    // Method to remove an invite from an attendee
    pub fn remove_invite_from_event(
        attendee_principal: Principal,
        event_identifier: Principal,
    ) -> Result<(), ApiError> {
        match Self::_get_attendee_from_caller(attendee_principal) {
            // if the attendee is not found, return an error
            None => Err(Self::_attendee_not_found_error(
                "remove_invite_from_attendee",
                None,
            )),
            // if the attendee is found, continue
            Some((_identifier, mut _attendee)) => {
                // filter out the event from the invites array
                let invites: Vec<Invite> = _attendee
                    .invites
                    .into_iter()
                    .filter(|j| &j.event_identifier != &event_identifier)
                    .collect();

                // set the invites array to the filtered array on the attendee
                _attendee.invites = invites;
                // update the attendee
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                Ok(())
            }
        }
    }

    // Method to add an invite or join to an attendee
    fn add_invite_or_join_event_to_attendee(
        caller: Principal,
        event_identifier: Principal,
        group_identifier: Principal,
        attendee: Option<(Principal, Attendee)>,
        event_privacy: Privacy,
    ) -> Result<Attendee, ApiError> {
        // Create the initial join entry
        let join = Join {
            event_identifier,
            group_identifier,
            updated_at: time(),
            created_at: time(),
        };

        // Create the initial invite entry
        let invite = Invite {
            event_identifier,
            group_identifier,
            invite_type: InviteType::UserRequest,
            updated_at: time(),
            created_at: time(),
        };

        use Privacy::*;
        match event_privacy {
            // if the event is public, add the join to the attendee
            Public => match attendee {
                // If the attendee is not found, create a new one and add the join to the joined array
                None => Ok(Attendee {
                    principal: caller,
                    joined: vec![join],
                    invites: vec![],
                }),
                // If the attendee is found, push the join to the existing joined array
                Some((_, mut _attendee)) => {
                    _attendee.joined.push(join);
                    Ok(_attendee)
                }
            },
            // if the event is private, add the invite to the attendee
            Private => match attendee {
                // If the attendee is not found, create a new one and add the invite to the invites array
                None => Ok(Attendee {
                    principal: caller,
                    joined: vec![],
                    invites: vec![invite],
                }),
                // If the attendee is found, push the invite to the existing invites array
                Some((_, mut _attendee)) => {
                    _attendee.invites.push(invite);
                    Ok(_attendee)
                }
            },
            // This method needs a different call to split the logic
            _ => {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "UNSUPPORTED",
                    "This type isnt supported through this call",
                    DATA.with(|data| Data::get_name(data)).as_str(),
                    "add_invite_or_join_event_to_attendee",
                    None,
                ))
            }
        }
    }

    // Method to get an attendee entry from the caller
    pub fn get_self(caller: Principal) -> Result<(Principal, Attendee), ApiError> {
        match Self::_get_attendee_from_caller(caller) {
            // if the attendee is not found, return an error
            None => Err(Self::_attendee_not_found_error("get_self", None)),
            // if the attendee is found, return the attendee
            Some(_attendee) => Ok(_attendee),
        }
    }

    // Method to get the event attendees from a single event
    pub fn get_event_attendees(event_identifier: Principal) -> Vec<JoinedAttendeeResponse> {
        DATA.with(|data| {
            let attendees = Data::get_entries(data);

            attendees
                .iter()
                .filter(|(_identifier, _attendee)| {
                    _attendee
                        .joined
                        .iter()
                        .any(|j| &j.event_identifier == &event_identifier)
                })
                .map(|(_identifier, _attendee)| {
                    Self::map_attendee_to_joined_attendee_response(
                        _identifier,
                        _attendee,
                        event_identifier.clone(),
                    )
                })
                .collect()
        })
    }

    // Method to get the event attendees count from multiple events
    pub fn get_event_attendees_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        // Create the initial attendees count array
        let mut attendees_count: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            // Get the attendees from the data canister
            let attendees = Data::get_entries(data);

            // Loop through the event identifiers
            for event_identifier in event_identifiers {
                let count = attendees
                    .iter()
                    // Filter the attendees to only those that have joined the event
                    .filter(|(_identifier, _attendee)| {
                        _attendee
                            .joined
                            .iter()
                            .any(|j| &j.event_identifier == &event_identifier)
                    })
                    .count();
                // Push the event identifier and the count to the attendees count array
                attendees_count.push((event_identifier, count));
            }
        });

        attendees_count
    }

    // Method to get the group invites from a single group
    pub fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        // Create the initial invite count array
        let mut attendees_count: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            // Get the attendees from the data canister
            let attendees = Data::get_entries(data);

            // Loop through the group identifiers
            for group_identifier in group_identifiers {
                // Get the count of attendees that have been invited to the group
                let count = attendees
                    .iter()
                    // Filter the attendees to only those that have been invited to the group
                    .filter(|(_identifier, attendee)| {
                        attendee
                            .invites
                            .iter()
                            .any(|j| &j.event_identifier == &group_identifier)
                    })
                    .count();
                // Push the group identifier and the count to the invite count array
                attendees_count.push((group_identifier, count));
            }
        });

        attendees_count
    }

    // Method to get the event invites from a single event
    pub fn get_event_invites(event_identifier: Principal) -> Vec<InviteAttendeeResponse> {
        DATA.with(|data| {
            // Get the attendees from the data canister
            Data::get_entries(data)
                .iter()
                // Filter the attendees to only those that have been invited to the event
                .filter(|(_identifier, _attendee)| {
                    _attendee
                        .invites
                        .iter()
                        .any(|j| &j.event_identifier == &event_identifier)
                })
                // Map the attendee to an invite attendee response
                .map(|(_identifier, _attendee)| {
                    Self::map_attendee_to_invite_attendee_response(
                        _identifier,
                        _attendee,
                        event_identifier.clone(),
                    )
                })
                .collect()
        })
    }

    // Method to invite
    pub fn invite_to_event(
        event_identifier: Principal,
        attendee_principal: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        // Create the initial invite
        let invite = Invite {
            event_identifier,
            invite_type: InviteType::OwnerRequest,
            group_identifier: group_identifier.clone(),
            updated_at: time(),
            created_at: time(),
        };

        match Self::_get_attendee_from_caller(attendee_principal) {
            // If the attendee is not found, create a new attendee and add the invite to the invites array
            None => {
                let attendee = Attendee {
                    principal: attendee_principal,
                    joined: vec![],
                    invites: vec![invite],
                };
                // Add the attendee to the data canister
                DATA.with(|data| Data::add_entry(data, attendee, Some("eae".to_string())))
            }
            // If the attendee is found
            Some((_identifier, mut _attendee)) => {
                // add the invite to the invites array
                _attendee.invites.push(invite);
                // Update the attendee in the data canister
                DATA.with(|data| Data::update_entry(data, _identifier, _attendee))
            }
        }
    }

    // Method to accept an invite as a admin
    pub fn accept_user_request_event_invite(
        attendee_principal: Principal,
        event_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        match Self::_get_attendee_from_caller(attendee_principal) {
            // If the attendee is not found, return an error
            None => Err(Self::_attendee_not_found_error(
                "accept_user_request_event_invite",
                None,
            )),
            // If the attendee is found, continue
            Some((_identifier, mut _attendee)) => {
                // Find the invite in the invites array
                let invite = _attendee
                    .invites
                    .iter()
                    .find(|i| &i.event_identifier == &event_identifier);

                match invite {
                    // If the invite is not found, return an error
                    None => Err(api_error(
                        ApiErrorType::NotFound,
                        "NO_INVITE_FOUND",
                        "There is no invite found for this event",
                        DATA.with(|data| Data::get_name(data)).as_str(),
                        "accept_user_request_event_invite",
                        None,
                    )),
                    // If the invite is found, continue
                    Some(_invite) => {
                        // If the invite type is not a user request, return an error
                        if _invite.invite_type != InviteType::UserRequest {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "INVALID_TYPE",
                                "Invalid invite type",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "accept_user_request_group_invite",
                                None,
                            ));
                        }
                        // Remove the invite from the invites array
                        _attendee.invites = _attendee
                            .invites
                            .into_iter()
                            .filter(|i| &i.event_identifier != &event_identifier)
                            .collect();

                        // Add the event to the joined array
                        _attendee.joined.push(Join {
                            event_identifier,
                            group_identifier: event_identifier,
                            updated_at: time(),
                            created_at: time(),
                        });

                        // Update the attendee in the data canister
                        let result =
                            DATA.with(|data| Data::update_entry(data, _identifier, _attendee));

                        // Update the attendee count on the event canister (fire-and-forget)
                        Self::update_attendee_count_on_event(&event_identifier);
                        result
                    }
                }
            }
        }
    }

    // Method to accept an invite as a user
    pub fn accept_owner_request_event_invite(
        caller: Principal,
        event_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        DATA.with(|data| {
            match Self::_get_attendee_from_caller(caller) {
                // If the attendee is not found, return an error
                None => Err(Self::_attendee_not_found_error(
                    "accept_owner_request_event_invite",
                    None,
                )),
                // If the attendee is found, continue
                Some((_identifier, mut _attendee)) => {
                    // Find the invite in the invites array
                    let invite = _attendee
                        .invites
                        .iter()
                        .find(|i| &i.event_identifier == &event_identifier);

                    match invite {
                        // If the invite is not found, return an error
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "NO_INVITE_FOUND",
                            "There is no invite found for this group",
                            Data::get_name(data).as_str(),
                            "accept_owner_request_event_invite",
                            None,
                        )),
                        // If the invite is found, continue
                        Some(_invite) => {
                            // If the invite type is not a owner request, return an error
                            if _invite.invite_type != InviteType::OwnerRequest {
                                return Err(api_error(
                                    ApiErrorType::BadRequest,
                                    "INVALID_TYPE",
                                    "Invalid invite type",
                                    Data::get_name(data).as_str(),
                                    "accept_owner_request_event_invite",
                                    None,
                                ));
                            }

                            // Remove the invite from the invites array
                            _attendee.invites = _attendee
                                .invites
                                .iter()
                                .filter(|i| &i.event_identifier == &event_identifier)
                                .cloned()
                                .collect();

                            // Add the event to the joined array
                            _attendee.joined.push(Join {
                                event_identifier,
                                updated_at: time(),
                                created_at: time(),
                                group_identifier: event_identifier,
                            });

                            // Update the attendee in the data canister
                            let response = Data::update_entry(data, _identifier, _attendee);
                            // Update the attendee count on the event canister (fire-and-forget)
                            Self::update_attendee_count_on_event(&event_identifier);
                            response
                        }
                    }
                }
            }
        })
    }

    // Method to get the event privacy and owner (inter-canister call)
    async fn _get_event_privacy_and_owner(
        event_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Privacy), ApiError> {
        // Call the get_event_privacy_and_owner method on the event canister
        let event_privacy_response: Result<(Result<(Principal, Privacy), ApiError>,), _> =
            call::call(
                Identifier::decode(&event_identifier).1,
                "get_event_privacy_and_owner",
                (event_identifier, group_identifier),
            )
            .await;

        DATA.with(|data| match event_privacy_response {
            // If the inter-canister call fails, return an error
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "INTER_CANISTER_CALL_FAILED",
                err.1.as_str(),
                Data::get_name(data).as_str(),
                "get_event_privacy_and_owner",
                None,
            )),
            // If the inter-canister call succeeds, continue and return the response
            Ok((_event_privacy,)) => match _event_privacy {
                Err(err) => Err(err),
                Ok(__event_privacy) => Ok(__event_privacy),
            },
        })
    }

    // Method used to map the attendee to a joined attendee response
    fn map_attendee_to_joined_attendee_response(
        identifier: &Principal,
        attendee: &Attendee,
        event_identifier: Principal,
    ) -> JoinedAttendeeResponse {
        JoinedAttendeeResponse {
            event_identifier,
            attendee_identifier: identifier.clone(),
            principal: attendee.principal,
            group_identifier: event_identifier,
        }
    }

    // Method used to map the attendee to an invite attendee response
    fn map_attendee_to_invite_attendee_response(
        identifier: &Principal,
        attendee: &Attendee,
        event_identifier: Principal,
    ) -> InviteAttendeeResponse {
        let invite = attendee
            .invites
            .iter()
            .find(|m| &m.event_identifier == &event_identifier);

        InviteAttendeeResponse {
            event_identifier,
            attendee_identifier: identifier.clone(),
            principal: attendee.principal,
            group_identifier: event_identifier,
            invite_type: match invite {
                None => InviteType::None,
                Some(_invite) => _invite.invite_type.clone(),
            },
        }
    }

    // Method to get the attendee from the caller principal
    fn _get_attendee_from_caller(caller: Principal) -> Option<(Principal, Attendee)> {
        let attendees = DATA.with(|data| Data::get_entries(data));
        attendees
            .into_iter()
            .find(|(_identifier, _attendee)| _attendee.principal == caller)
    }

    // Method to get the attendee count for an event
    fn _get_attendee_count_for_event(group_identifier: &Principal) -> usize {
        let attendees = DATA.with(|data| Data::get_entries(data));
        attendees
            .iter()
            .filter(|(_identifier, _attendee)| {
                _attendee
                    .joined
                    .iter()
                    .any(|j| &j.event_identifier == group_identifier)
            })
            .count()
    }

    // Default error for when an attendee is not found
    fn _attendee_not_found_error(method_name: &str, inputs: Option<Vec<String>>) -> ApiError {
        api_error(
            ApiErrorType::NotFound,
            "ATTENDEE_NOT_FOUND",
            "Attendee not found",
            DATA.with(|data| Data::get_name(data)).as_str(),
            method_name,
            inputs,
        )
    }

    // Method to add the owner of an event as an attendee
    pub fn add_owner_as_attendee(
        user_principal: Principal,
        event_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(), bool> {
        let attendee = Self::_get_attendee_from_caller(user_principal);

        // Decode the event and group identifiers and see if they are valid
        let (_, _event_canister, _event_kind) = Identifier::decode(&event_identifier);
        let (_, _, _group_kind) = Identifier::decode(&group_identifier);

        // check if it is an event identifier
        if _event_kind != "evt" {
            return Err(false);
        }

        // check if it is a group identifier
        if _group_kind != "grp" {
            return Err(false);
        }

        // Check if the caller is the event canister
        if caller() != _event_canister {
            return Err(false);
        }

        // Create the intial join object
        let join = Join {
            event_identifier: event_identifier.clone(),
            created_at: time(),
            updated_at: time(),
            group_identifier,
        };

        match attendee {
            // If the attendee does not exist, create a new attendee and add the join to
            None => {
                let attendee = Attendee {
                    principal: user_principal,
                    joined: vec![join],
                    invites: vec![],
                };
                // Add the attendee to the attendees
                let _ = DATA.with(|data| Data::add_entry(data, attendee, Some("eae".to_string())));
                Ok(())
            }
            // If the attendee exists, continue
            Some((_identifier, mut _attendee)) => {
                // If the attendee has already joined the event, return an error
                if _attendee
                    .joined
                    .iter()
                    .any(|j| &j.event_identifier == &event_identifier)
                {
                    return Err(true);
                    // If the attendee has not joined the event, add the join to the attendee
                } else {
                    _attendee.joined.push(join);
                    // Update the attendee
                    let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                    return Ok(());
                }
            }
        }
    }

    // Method to update the attendee count on the event
    #[allow(unused_must_use)]
    fn update_attendee_count_on_event(event_identifier: &Principal) -> () {
        // Get the attendee count for the event
        let event_attendees_count_array =
            Self::get_event_attendees_count(vec![event_identifier.clone()]);

        // Set the initial count to 0
        let mut count = 0;

        // If the attendee count array is not empty, set the count to the first element
        if event_attendees_count_array.len() > 0 {
            count = event_attendees_count_array[0].1;
        };

        // Decode the event identifier and call the update attendee count method on the event
        let (_, event_canister, _) = Identifier::decode(event_identifier);
        call::call::<(Principal, Principal, usize), ()>(
            event_canister,
            "update_attendee_count_on_event",
            (event_identifier.clone(), id(), count),
        );
    }

    // This method is used for role / permission based access control
    pub async fn can_write(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Write,
            PermissionType::Event(None),
        )
        .await
    }

    // This method is used for role / permission based access control
    pub async fn can_read(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Read,
            PermissionType::Event(None),
        )
        .await
    }

    // This method is used for role / permission based access control
    pub async fn can_edit(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Edit,
            PermissionType::Event(None),
        )
        .await
    }

    // This method is used for role / permission based access control
    pub async fn can_delete(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
    ) -> Result<Principal, ApiError> {
        Self::check_permission(
            caller,
            group_identifier,
            member_identifier,
            PermissionActionType::Delete,
            PermissionType::Event(None),
        )
        .await
    }

    // This method is used for role / permission based access control
    async fn check_permission(
        caller: Principal,
        group_identifier: Principal,
        member_identifier: Principal,
        permission: PermissionActionType,
        permission_type: PermissionType,
    ) -> Result<Principal, ApiError> {
        let group_roles = get_group_roles(group_identifier).await;
        let member_roles = get_member_roles(member_identifier, group_identifier).await;

        match member_roles {
            Ok((_principal, _roles)) => {
                if caller != _principal {
                    return Err(api_error(
                        ApiErrorType::Unauthorized,
                        "PRINCIPAL_MISMATCH",
                        "Principal mismatch",
                        DATA.with(|data| Data::get_name(data)).as_str(),
                        "check_permission",
                        None,
                    ));
                }

                match group_roles {
                    Ok(mut _group_roles) => {
                        _group_roles.append(&mut default_roles());
                        let has_permission =
                            has_permission(&_roles, &permission_type, &_group_roles, &permission);

                        if !has_permission {
                            return Err(api_error(
                                ApiErrorType::Unauthorized,
                                "NO_PERMISSION",
                                "No permission",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "check_permission",
                                None,
                            ));
                        }

                        Ok(caller)
                    }
                    Err(err) => Err(api_error(
                        ApiErrorType::Unauthorized,
                        "NO_PERMISSION",
                        err.as_str(),
                        DATA.with(|data| Data::get_name(data)).as_str(),
                        "check_permission",
                        None,
                    )),
                }
            }
            Err(err) => Err(api_error(
                ApiErrorType::Unauthorized,
                "NO_PERMISSION",
                err.as_str(),
                DATA.with(|data| Data::get_name(data)).as_str(),
                "check_permission",
                None,
            )),
        }
    }

    // Used for composite_query calls from the parent canister
    //
    // Method to get filtered attendees serialized and chunked
    pub fn get_chunked_join_data(
        event_identifier: &Principal,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let attendees = DATA.with(|data| Data::get_entries(data));
        // Get attendees for filtering
        let mapped_attendees: Vec<JoinedAttendeeResponse> = attendees
            .iter()
            // Filter attendees that have joined the group
            .filter(|(_identifier, _attendee_data)| {
                _attendee_data
                    .joined
                    .iter()
                    .any(|j| &j.event_identifier == event_identifier)
            })
            // Map attendee to joined attendee response
            .map(|(_identifier, _attendee_data)| {
                Self::map_attendee_to_joined_attendee_response(
                    _identifier,
                    _attendee_data,
                    event_identifier.clone(),
                )
            })
            .collect();

        if let Ok(bytes) = serialize(&mapped_attendees) {
            // Check if the bytes of the serialized groups are greater than the max bytes per chunk specified as an argument
            if bytes.len() >= max_bytes_per_chunk {
                // Get the start and end index of the bytes to be returned
                let start = chunk * max_bytes_per_chunk;
                let end = (chunk + 1) * (max_bytes_per_chunk);

                // Get the bytes to be returned, if the end index is greater than the length of the bytes, return the remaining bytes
                let response = if end >= bytes.len() {
                    bytes[start..].to_vec()
                } else {
                    bytes[start..end].to_vec()
                };

                // Determine the max number of chunks that can be returned, a float is used because the number of chunks can be a decimal in this step
                let mut max_chunks: f64 = 0.00;
                if max_bytes_per_chunk < bytes.len() {
                    max_chunks = (bytes.len() / max_bytes_per_chunk) as f64;
                }

                // return the response and start and end chunk index, the end chunk index is calculated by rounding up the max chunks
                return (response, (chunk, max_chunks.ceil() as usize));
            }

            // if the bytes of the serialized groups are less than the max bytes per chunk specified as an argument, return the bytes and start and end chunk index as 0
            return (bytes, (0, 0));
        } else {
            // if the groups cant be serialized return an empty vec and start and end chunk index as 0
            return (vec![], (0, 0));
        }
    }

    // Used for composite_query calls from the parent canister
    //
    // Method to get filtered attendees serialized and chunked
    pub fn get_chunked_invite_data(
        event_identifier: &Principal,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let attendees = DATA.with(|data| Data::get_entries(data));
        // Get attendees for filtering
        let mapped_attendees: Vec<InviteAttendeeResponse> = attendees
            .iter()
            // Filter attendees that have joined the group
            .filter(|(_identifier, _attendee_data)| {
                _attendee_data
                    .invites
                    .iter()
                    .any(|j| &j.event_identifier == event_identifier)
            })
            // Map member to joined member response
            .map(|(_identifier, _event_data)| {
                Self::map_attendee_to_invite_attendee_response(
                    _identifier,
                    _event_data,
                    event_identifier.clone(),
                )
            })
            .collect();

        if let Ok(bytes) = serialize(&mapped_attendees) {
            // Check if the bytes of the serialized groups are greater than the max bytes per chunk specified as an argument
            if bytes.len() >= max_bytes_per_chunk {
                // Get the start and end index of the bytes to be returned
                let start = chunk * max_bytes_per_chunk;
                let end = (chunk + 1) * (max_bytes_per_chunk);

                // Get the bytes to be returned, if the end index is greater than the length of the bytes, return the remaining bytes
                let response = if end >= bytes.len() {
                    bytes[start..].to_vec()
                } else {
                    bytes[start..end].to_vec()
                };

                // Determine the max number of chunks that can be returned, a float is used because the number of chunks can be a decimal in this step
                let mut max_chunks: f64 = 0.00;
                if max_bytes_per_chunk < bytes.len() {
                    max_chunks = (bytes.len() / max_bytes_per_chunk) as f64;
                }

                // return the response and start and end chunk index, the end chunk index is calculated by rounding up the max chunks
                return (response, (chunk, max_chunks.ceil() as usize));
            }

            // if the bytes of the serialized groups are less than the max bytes per chunk specified as an argument, return the bytes and start and end chunk index as 0
            return (bytes, (0, 0));
        } else {
            // if the groups cant be serialized return an empty vec and start and end chunk index as 0
            return (vec![], (0, 0));
        }
    }
}
