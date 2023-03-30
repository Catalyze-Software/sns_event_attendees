import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type ApiError = { 'SerializeError' : ErrorMessage } |
  { 'DeserializeError' : ErrorMessage } |
  { 'NotFound' : ErrorMessage } |
  { 'ValidationError' : Array<ValidationResponse> } |
  { 'CanisterAtCapacity' : ErrorMessage } |
  { 'UpdateRequired' : UpdateMessage } |
  { 'Unauthorized' : ErrorMessage } |
  { 'Unexpected' : ErrorMessage } |
  { 'BadRequest' : ErrorMessage };
export interface ErrorMessage {
  'tag' : string,
  'message' : string,
  'inputs' : [] | [Array<string>],
  'location' : string,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface Invite {
  'updated_at' : bigint,
  'group_identifier' : Principal,
  'invite_type' : InviteType,
  'created_at' : bigint,
}
export interface InviteMemberResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'invite' : Invite,
  'member_identifier' : Principal,
}
export type InviteType = { 'OwnerRequest' : null } |
  { 'UserRequest' : null };
export interface Join {
  'updated_at' : bigint,
  'group_identifier' : Principal,
  'created_at' : bigint,
  'roles' : Array<string>,
}
export interface JoinedMemberResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'member_identifier' : Principal,
  'roles' : Array<string>,
}
export interface Member {
  'principal' : Principal,
  'invites' : Array<Invite>,
  'joined' : Array<Join>,
  'profile_identifier' : Principal,
}
export interface Metadata {
  'updated_at' : bigint,
  'name' : string,
  'max_entries' : bigint,
  'current_entry_id' : [] | [bigint],
  'created_at' : bigint,
  'used_data' : bigint,
  'cycles' : bigint,
  'is_available' : boolean,
  'identifier' : bigint,
  'entries_count' : bigint,
  'parent' : Principal,
}
export type Result = { 'Ok' : [Principal, Member] } |
  { 'Err' : ApiError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : ApiError };
export type Result_2 = { 'Ok' : Principal } |
  { 'Err' : ApiError };
export type Result_3 = { 'Ok' : null } |
  { 'Err' : null };
export type Result_4 = { 'Ok' : Array<InviteMemberResponse> } |
  { 'Err' : ApiError };
export type Result_5 = { 'Ok' : JoinedMemberResponse } |
  { 'Err' : ApiError };
export type Result_6 = { 'Ok' : Array<JoinedMemberResponse> } |
  { 'Err' : ApiError };
export type Result_7 = { 'Ok' : [Principal, Array<string>] } |
  { 'Err' : string };
export type Result_8 = { 'Ok' : Metadata } |
  { 'Err' : ApiError };
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'accept_owner_request_group_invite' : ActorMethod<[Principal], Result>,
  'accept_user_request_group_invite' : ActorMethod<
    [Principal, Principal],
    Result
  >,
  'add_entry_by_parent' : ActorMethod<[Uint8Array | number[]], Result_1>,
  'add_owner' : ActorMethod<[Principal, Principal], Result_2>,
  'assign_role' : ActorMethod<[string, Principal, Principal], Result_3>,
  'create_empty_member' : ActorMethod<[Principal, Principal], Result_2>,
  'get_group_invites' : ActorMethod<[Principal], Result_4>,
  'get_group_invites_count' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, bigint]>
  >,
  'get_group_member' : ActorMethod<[Principal, Principal], Result_5>,
  'get_group_members' : ActorMethod<[Principal], Result_6>,
  'get_group_members_count' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, bigint]>
  >,
  'get_groups_for_members' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, Array<Principal>]>
  >,
  'get_member_roles' : ActorMethod<[Principal, Principal], Result_7>,
  'get_metadata' : ActorMethod<[], Result_8>,
  'get_self' : ActorMethod<[], Result>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'invite_to_group' : ActorMethod<[Principal, Principal], Result>,
  'join_group' : ActorMethod<[Principal, [] | [string]], Result>,
  'leave_group' : ActorMethod<[Principal], Result_1>,
  'remove_invite' : ActorMethod<[Principal], Result_1>,
  'remove_member_from_group' : ActorMethod<[Principal, Principal], Result_1>,
  'remove_member_invite_from_group' : ActorMethod<
    [Principal, Principal],
    Result_1
  >,
  'remove_role' : ActorMethod<[string, Principal, Principal], Result_3>,
  'sanity_check' : ActorMethod<[], string>,
}
