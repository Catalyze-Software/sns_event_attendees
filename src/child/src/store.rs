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
            Ok((_, _event_privacy)) => {
                let existing_attendee = Self::_get_attendee_from_caller(caller);

                match existing_attendee.clone() {
                    // If there is no exisiting attendee
                    None => {}
                    Some((_identifier, _exisiting_attendee)) => {
                        if _exisiting_attendee.principal != caller {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "UNAUTHORIZED",
                                "You are not authorized to perform this action",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "join_event",
                                None,
                            ));
                        }
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

                let updated_attendee = Self::add_invite_or_join_event_to_attendee(
                    caller,
                    event_identifier.clone(),
                    group_identifier,
                    existing_attendee.clone(),
                    _event_privacy,
                );

                match updated_attendee {
                    Err(err) => Err(err),
                    Ok(_updated_attendee) => match existing_attendee {
                        None => {
                            let result = DATA.with(|data| {
                                Data::add_entry(data, _updated_attendee, Some("mbr".to_string()))
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
                // add scaling logic
                // Determine if an entry needs to be updated or added as a new one
            }
        }
    }

    pub fn leave_event(caller: Principal, event_identifier: Principal) -> Result<(), ApiError> {
        let existing_attendee = Self::_get_attendee_from_caller(caller);

        match existing_attendee {
            None => Err(Self::_attendee_not_found_error("leave_event", None)),
            Some((_identifier, mut _attendee)) => {
                let joined: Vec<Join> = _attendee
                    .joined
                    .into_iter()
                    .filter(|j| &j.event_identifier != &event_identifier)
                    .collect();

                _attendee.joined = joined;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                Ok(Self::update_attendee_count_on_event(&event_identifier))
            }
        }
    }

    pub fn remove_invite(caller: Principal, event_identifier: Principal) -> Result<(), ApiError> {
        let existing_attendee = Self::_get_attendee_from_caller(caller);
        match existing_attendee {
            None => Err(Self::_attendee_not_found_error("remove_invite", None)),
            Some((_identifier, mut _attendee)) => {
                let invites: Vec<Invite> = _attendee
                    .invites
                    .into_iter()
                    .filter(|j| &j.event_identifier != &event_identifier)
                    .collect();

                _attendee.invites = invites;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                Ok(())
            }
        }
    }

    pub fn remove_join_from_attendee(
        attendee_principal: Principal,
        event_identifier: Principal,
    ) -> Result<(), ApiError> {
        match Self::_get_attendee_from_caller(attendee_principal) {
            None => Err(Self::_attendee_not_found_error(
                "remove_join_from_attendee",
                None,
            )),
            Some((_identifier, mut _attendee)) => {
                let joined: Vec<Join> = _attendee
                    .joined
                    .into_iter()
                    .filter(|j| &j.event_identifier != &event_identifier)
                    .collect();

                _attendee.joined = joined;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                return Ok(Self::update_attendee_count_on_event(&event_identifier));
            }
        }
    }

    pub fn remove_invite_from_event(
        attendee_principal: Principal,
        event_identifier: Principal,
    ) -> Result<(), ApiError> {
        let existing = Self::_get_attendee_from_caller(attendee_principal);
        match existing {
            None => Err(Self::_attendee_not_found_error(
                "remove_invite_from_attendee",
                None,
            )),
            Some((_identifier, mut _attendee)) => {
                let invites: Vec<Invite> = _attendee
                    .invites
                    .into_iter()
                    .filter(|j| &j.event_identifier != &event_identifier)
                    .collect();

                _attendee.invites = invites;
                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                Ok(())
            }
        }
    }

    fn add_invite_or_join_event_to_attendee(
        caller: Principal,
        event_identifier: Principal,
        group_identifier: Principal,
        attendee: Option<(Principal, Attendee)>,
        event_privacy: Privacy,
    ) -> Result<Attendee, ApiError> {
        let join = Join {
            event_identifier,
            group_identifier,
            updated_at: time(),
            created_at: time(),
        };

        let invite = Invite {
            event_identifier,
            group_identifier,
            invite_type: InviteType::UserRequest,
            updated_at: time(),
            created_at: time(),
        };

        use Privacy::*;
        match event_privacy {
            // Create a joined entry based on the group privacy settings
            Public => match attendee {
                None => Ok(Attendee {
                    principal: caller,
                    joined: vec![join],
                    invites: vec![],
                }),
                Some((_, mut _attendee)) => {
                    _attendee.joined.push(join);
                    Ok(_attendee)
                }
            },
            // Create a invite entry based on the group privacy settings
            Private => match attendee {
                None => Ok(Attendee {
                    principal: caller,
                    joined: vec![],
                    invites: vec![invite],
                }),
                Some((_, mut _attendee)) => {
                    _attendee.invites.push(invite);
                    Ok(_attendee)
                }
            },
            // This method needs a different call to split the logic
            InviteOnly => {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "UNSUPPORTED",
                    "This type of invite isnt supported through this call",
                    DATA.with(|data| Data::get_name(data)).as_str(),
                    "add_invite_or_join_event_to_attendee",
                    None,
                ))
            }
            Gated(_) => {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "UNSUPPORTED",
                    "This type of invite isnt supported through this call",
                    DATA.with(|data| Data::get_name(data)).as_str(),
                    "add_invite_or_join_event_to_attendee",
                    None,
                ))
            }
        }
    }

    pub fn get_self(caller: Principal) -> Result<(Principal, Attendee), ApiError> {
        let existing = Self::_get_attendee_from_caller(caller);
        match existing {
            None => Err(Self::_attendee_not_found_error("get_self", None)),
            Some(_attendee) => Ok(_attendee),
        }
    }

    // pub fn get_event_attendee_by_principal(
    //     caller: Principal,
    //     event_identifier: Principal,
    // ) -> Result<JoinedAttendeeResponse, ApiError> {
    //     DATA.with(|data| {
    //         let existing_attendee = Store::_get_attendee_from_caller(caller);
    //         match existing_attendee {
    //             None => Err(Self::_attendee_not_found_error("get_self", None)),
    //             Some((_identifier, _attendee)) => {
    //                 let join = _attendee
    //                     .joined
    //                     .iter()
    //                     .find(|j| &j.event_identifier == &event_identifier);

    //                 match join {
    //                     None => Err(api_error(
    //                         ApiErrorType::NotFound,
    //                         "NOT_JOINED",
    //                         "Not an attendee",
    //                         Data::get_name(data).as_str(),
    //                         "get_event_attendee_by_principal",
    //                         None,
    //                     )),
    //                     Some(_join) => Ok(JoinedAttendeeResponse {
    //                         event_identifier: _join.event_identifier,
    //                         group_identifier: _join.group_identifier,
    //                         attendee_identifier: _identifier,
    //                         principal: caller,
    //                     }),
    //                 }
    //             }
    //         }
    //     })
    // }

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

    pub fn get_event_attendees_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        let mut attendees_count: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            let attendees = Data::get_entries(data);

            for event_identifier in event_identifiers {
                let count = attendees
                    .iter()
                    .filter(|(_identifier, _attendee)| {
                        _attendee
                            .joined
                            .iter()
                            .any(|j| &j.event_identifier == &event_identifier)
                    })
                    .count();
                attendees_count.push((event_identifier, count));
            }
        });

        attendees_count
    }

    pub fn get_group_invites_count(group_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {
        let mut attendees_count: Vec<(Principal, usize)> = vec![];

        DATA.with(|data| {
            let attendees = Data::get_entries(data);

            for group_identifier in group_identifiers {
                let count = attendees
                    .iter()
                    .filter(|(_identifier, attendee)| {
                        attendee
                            .invites
                            .iter()
                            .any(|j| &j.event_identifier == &group_identifier)
                    })
                    .count();
                attendees_count.push((group_identifier, count));
            }
        });

        attendees_count
    }

    pub fn get_event_invites(event_identifier: Principal) -> Vec<InviteAttendeeResponse> {
        DATA.with(|data| {
            let attendees = Data::get_entries(data);

            attendees
                .iter()
                .filter(|(_identifier, _attendee)| {
                    _attendee
                        .invites
                        .iter()
                        .any(|j| &j.event_identifier == &event_identifier)
                })
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

    pub fn invite_to_event(
        event_identifier: Principal,
        attendee_principal: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        let exisiting_attendee = Self::_get_attendee_from_caller(attendee_principal);

        let invite = Invite {
            event_identifier,
            invite_type: InviteType::OwnerRequest,
            group_identifier: group_identifier.clone(),
            updated_at: time(),
            created_at: time(),
        };

        match exisiting_attendee {
            None => {
                let attendee = Attendee {
                    principal: attendee_principal,
                    joined: vec![],
                    invites: vec![invite],
                };
                DATA.with(|data| Data::add_entry(data, attendee, Some("mbr".to_string())))
            }
            Some((_identifier, mut _attendee)) => {
                _attendee.invites.push(invite);
                DATA.with(|data| Data::update_entry(data, _identifier, _attendee))
            }
        }
    }

    pub fn accept_user_request_event_invite(
        attendee_principal: Principal,
        event_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        let attendee = Self::_get_attendee_from_caller(attendee_principal);

        match attendee {
            None => Err(Self::_attendee_not_found_error(
                "accept_user_request_event_invite",
                None,
            )),
            Some((_identifier, mut _attendee)) => {
                let invite = _attendee
                    .invites
                    .iter()
                    .find(|i| &i.event_identifier == &event_identifier);

                match invite {
                    None => Err(api_error(
                        ApiErrorType::NotFound,
                        "NO_INVITE_FOUND",
                        "There is no invite found for this event",
                        DATA.with(|data| Data::get_name(data)).as_str(),
                        "accept_user_request_event_invite",
                        None,
                    )),
                    Some(_invite) => {
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

                        _attendee.invites = _attendee
                            .invites
                            .into_iter()
                            .filter(|i| &i.event_identifier != &event_identifier)
                            .collect();

                        _attendee.joined.push(Join {
                            event_identifier,
                            group_identifier: event_identifier,
                            updated_at: time(),
                            created_at: time(),
                        });

                        let result =
                            DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                        Self::update_attendee_count_on_event(&event_identifier);
                        result
                    }
                }
            }
        }
    }

    pub fn accept_owner_request_event_invite(
        caller: Principal,
        event_identifier: Principal,
    ) -> Result<(Principal, Attendee), ApiError> {
        DATA.with(|data| {
            let existing_attendee = Self::_get_attendee_from_caller(caller);

            match existing_attendee {
                None => Err(Self::_attendee_not_found_error(
                    "accept_owner_request_event_invite",
                    None,
                )),
                Some((_identifier, mut _attendee)) => {
                    let invite = _attendee
                        .invites
                        .iter()
                        .find(|i| &i.event_identifier == &event_identifier);
                    match invite {
                        None => Err(api_error(
                            ApiErrorType::NotFound,
                            "NO_INVITE_FOUND",
                            "There is no invite found for this group",
                            Data::get_name(data).as_str(),
                            "accept_owner_request_event_invite",
                            None,
                        )),
                        Some(_invite) => {
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

                            _attendee.invites = _attendee
                                .invites
                                .iter()
                                .filter(|i| &i.event_identifier == &event_identifier)
                                .cloned()
                                .collect();

                            _attendee.joined.push(Join {
                                event_identifier,
                                updated_at: time(),
                                created_at: time(),
                                group_identifier: event_identifier,
                            });
                            let response = Data::update_entry(data, _identifier, _attendee);
                            Self::update_attendee_count_on_event(&event_identifier);
                            response
                        }
                    }
                }
            }
        })
    }

    async fn _get_event_privacy_and_owner(
        event_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(Principal, Privacy), ApiError> {
        let event_privacy_response: Result<(Result<(Principal, Privacy), ApiError>,), _> =
            call::call(
                Identifier::decode(&event_identifier).1,
                "get_event_privacy_and_owner",
                (event_identifier, group_identifier),
            )
            .await;

        DATA.with(|data| match event_privacy_response {
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "INTER_CANISTER_CALL_FAILED",
                err.1.as_str(),
                Data::get_name(data).as_str(),
                "get_event_privacy_and_owner",
                None,
            )),
            Ok((_event_privacy,)) => match _event_privacy {
                Err(err) => Err(err),
                Ok(__event_privacy) => Ok(__event_privacy),
            },
        })
    }

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

    fn _get_attendee_from_caller(caller: Principal) -> Option<(Principal, Attendee)> {
        let attendees = DATA.with(|data| Data::get_entries(data));
        attendees
            .into_iter()
            .find(|(_identifier, _attendee)| _attendee.principal == caller)
    }

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

    pub fn add_owner_as_attendee(
        user_principal: Principal,
        event_identifier: Principal,
        group_identifier: Principal,
    ) -> Result<(), bool> {
        let attendee = Self::_get_attendee_from_caller(user_principal);

        let (_, _event_canister, _event_kind) = Identifier::decode(&event_identifier);
        let (_, _, _group_kind) = Identifier::decode(&group_identifier);

        if _event_kind != "evt" {
            return Err(false);
        }

        if _group_kind != "grp" {
            return Err(false);
        }

        if caller() != _event_canister {
            return Err(false);
        }

        let join = Join {
            event_identifier: event_identifier.clone(),
            created_at: time(),
            updated_at: time(),
            group_identifier,
        };

        match attendee {
            None => {
                let attendee = Attendee {
                    principal: user_principal,
                    joined: vec![join],
                    invites: vec![],
                };
                let _ = DATA.with(|data| Data::add_entry(data, attendee, Some("mbr".to_string())));
                Ok(())
            }
            Some((_identifier, mut _attendee)) => {
                if _attendee
                    .joined
                    .iter()
                    .any(|j| &j.event_identifier == &event_identifier)
                {
                    return Err(true);
                } else {
                    _attendee.joined.push(join);
                    let _ = DATA.with(|data| Data::update_entry(data, _identifier, _attendee));
                    return Ok(());
                }
            }
        }
    }

    #[allow(unused_must_use)]
    fn update_attendee_count_on_event(event_identifier: &Principal) -> () {
        let event_attendees_count_array =
            Self::get_event_attendees_count(vec![event_identifier.clone()]);
        let mut count = 0;

        if event_attendees_count_array.len() > 0 {
            count = event_attendees_count_array[0].1;
        };

        let (_, event_canister, _) = Identifier::decode(event_identifier);
        call::call::<(Principal, Principal, usize), ()>(
            event_canister,
            "update_attendee_count_on_event",
            (event_identifier.clone(), id(), count),
        );
    }

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
}
