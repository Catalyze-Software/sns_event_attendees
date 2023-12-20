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
export interface Attendee {
  'principal' : Principal,
  'invites' : Array<[Principal, Invite]>,
  'joined' : Array<[Principal, Join]>,
}
export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
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
export interface InviteAttendeeResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'attendee_identifier' : Principal,
  'invite_type' : InviteType,
  'event_identifier' : Principal,
}
export type InviteType = { 'None' : null } |
  { 'OwnerRequest' : null } |
  { 'UserRequest' : null };
export interface Join {
  'updated_at' : bigint,
  'group_identifier' : Principal,
  'created_at' : bigint,
}
export interface JoinedAttendeeResponse {
  'principal' : Principal,
  'group_identifier' : Principal,
  'attendee_identifier' : Principal,
  'event_identifier' : Principal,
}
export type RejectionCode = { 'NoError' : null } |
  { 'CanisterError' : null } |
  { 'SysTransient' : null } |
  { 'DestinationInvalid' : null } |
  { 'Unknown' : null } |
  { 'SysFatal' : null } |
  { 'CanisterReject' : null };
export type Result = { 'Ok' : [Principal, Attendee] } |
  { 'Err' : ApiError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : ApiError };
export type Result_2 = { 'Ok' : null } |
  { 'Err' : boolean };
export type Result_3 = { 'Ok' : [CanisterStatusResponse] } |
  { 'Err' : [RejectionCode, string] };
export type Result_4 = { 'Ok' : Array<JoinedAttendeeResponse> } |
  { 'Err' : ApiError };
export type Result_5 = { 'Ok' : Array<InviteAttendeeResponse> } |
  { 'Err' : ApiError };
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'accept_owner_request_event_invite' : ActorMethod<[Principal], Result>,
  'accept_user_request_event_invite' : ActorMethod<
    [Principal, Principal, Principal, Principal],
    Result
  >,
  'add_entry_by_parent' : ActorMethod<[Uint8Array | number[]], Result_1>,
  'add_owner_as_attendee' : ActorMethod<
    [Principal, Principal, Principal],
    Result_2
  >,
  'canister_backup_data' : ActorMethod<[], [string, string]>,
  'canister_status' : ActorMethod<[], Result_3>,
  'clear_backup' : ActorMethod<[], undefined>,
  'download_chunk' : ActorMethod<[bigint], [bigint, Uint8Array | number[]]>,
  'download_entries_chunk' : ActorMethod<
    [bigint],
    [bigint, Uint8Array | number[]]
  >,
  'download_stable_data_chunk' : ActorMethod<
    [bigint],
    [bigint, Uint8Array | number[]]
  >,
  'finalize_upload' : ActorMethod<[], string>,
  'get_attending_from_principal' : ActorMethod<[Principal], Result_4>,
  'get_chunked_invite_data' : ActorMethod<
    [Principal, bigint, bigint],
    [Uint8Array | number[], [bigint, bigint]]
  >,
  'get_chunked_join_data' : ActorMethod<
    [Principal, bigint, bigint],
    [Uint8Array | number[], [bigint, bigint]]
  >,
  'get_event_attendees' : ActorMethod<[Principal], Result_4>,
  'get_event_attendees_count' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, bigint]>
  >,
  'get_event_invites' : ActorMethod<
    [Principal, Principal, Principal],
    Result_5
  >,
  'get_event_invites_count' : ActorMethod<
    [Array<Principal>],
    Array<[Principal, bigint]>
  >,
  'get_self' : ActorMethod<[], Result>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'invite_to_event' : ActorMethod<
    [Principal, Principal, Principal, Principal],
    Result
  >,
  'join_event' : ActorMethod<[Principal, Principal], Result>,
  'leave_event' : ActorMethod<[Principal], Result_1>,
  'remove_attendee_from_event' : ActorMethod<
    [Principal, Principal, Principal, Principal],
    Result_1
  >,
  'remove_attendee_invite_from_event' : ActorMethod<
    [Principal, Principal, Principal, Principal],
    Result_1
  >,
  'remove_invite' : ActorMethod<[Principal], Result_1>,
  'restore_data' : ActorMethod<[], undefined>,
  'sanity_check' : ActorMethod<[], string>,
  'total_chunks' : ActorMethod<[], bigint>,
  'total_entries_chunks' : ActorMethod<[], bigint>,
  'total_stable_data_chunks' : ActorMethod<[], bigint>,
  'upload_chunk' : ActorMethod<[[bigint, Uint8Array | number[]]], undefined>,
}
