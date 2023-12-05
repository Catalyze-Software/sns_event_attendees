export const idlFactory = ({ IDL }) => {
  const InviteType = IDL.Variant({
    'None' : IDL.Null,
    'OwnerRequest' : IDL.Null,
    'UserRequest' : IDL.Null,
  });
  const Invite = IDL.Record({
    'updated_at' : IDL.Nat64,
    'group_identifier' : IDL.Principal,
    'invite_type' : InviteType,
    'created_at' : IDL.Nat64,
  });
  const Join = IDL.Record({
    'updated_at' : IDL.Nat64,
    'group_identifier' : IDL.Principal,
    'created_at' : IDL.Nat64,
  });
  const Attendee = IDL.Record({
    'principal' : IDL.Principal,
    'invites' : IDL.Vec(IDL.Tuple(IDL.Principal, Invite)),
    'joined' : IDL.Vec(IDL.Tuple(IDL.Principal, Join)),
  });
  const ErrorMessage = IDL.Record({
    'tag' : IDL.Text,
    'message' : IDL.Text,
    'inputs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'location' : IDL.Text,
  });
  const ValidationResponse = IDL.Record({
    'field' : IDL.Text,
    'message' : IDL.Text,
  });
  const UpdateMessage = IDL.Record({
    'canister_principal' : IDL.Principal,
    'message' : IDL.Text,
  });
  const ApiError = IDL.Variant({
    'SerializeError' : ErrorMessage,
    'DeserializeError' : ErrorMessage,
    'NotFound' : ErrorMessage,
    'ValidationError' : IDL.Vec(ValidationResponse),
    'CanisterAtCapacity' : ErrorMessage,
    'UpdateRequired' : UpdateMessage,
    'Unauthorized' : ErrorMessage,
    'Unexpected' : ErrorMessage,
    'BadRequest' : ErrorMessage,
  });
  const Result = IDL.Variant({
    'Ok' : IDL.Tuple(IDL.Principal, Attendee),
    'Err' : ApiError,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : ApiError });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Bool });
  const JoinedAttendeeResponse = IDL.Record({
    'principal' : IDL.Principal,
    'group_identifier' : IDL.Principal,
    'attendee_identifier' : IDL.Principal,
    'event_identifier' : IDL.Principal,
  });
  const Result_3 = IDL.Variant({
    'Ok' : IDL.Vec(JoinedAttendeeResponse),
    'Err' : ApiError,
  });
  const InviteAttendeeResponse = IDL.Record({
    'principal' : IDL.Principal,
    'group_identifier' : IDL.Principal,
    'attendee_identifier' : IDL.Principal,
    'invite_type' : InviteType,
    'event_identifier' : IDL.Principal,
  });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Vec(InviteAttendeeResponse),
    'Err' : ApiError,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  return IDL.Service({
    '__get_candid_interface_tmp_hack' : IDL.Func([], [IDL.Text], ['query']),
    'accept_cycles' : IDL.Func([], [IDL.Nat64], []),
    'accept_owner_request_event_invite' : IDL.Func(
        [IDL.Principal],
        [Result],
        [],
      ),
    'accept_user_request_event_invite' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal, IDL.Principal],
        [Result],
        [],
      ),
    'add_entry_by_parent' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result_1], []),
    'add_owner_as_attendee' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal],
        [Result_2],
        [],
      ),
    'clear_backup' : IDL.Func([], [], []),
    'download_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'finalize_upload' : IDL.Func([], [IDL.Text], []),
    'get_attending_from_principal' : IDL.Func(
        [IDL.Principal],
        [Result_3],
        ['query'],
      ),
    'get_chunked_invite_data' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Nat8), IDL.Tuple(IDL.Nat64, IDL.Nat64)],
        ['query'],
      ),
    'get_chunked_join_data' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Nat8), IDL.Tuple(IDL.Nat64, IDL.Nat64)],
        ['query'],
      ),
    'get_event_attendees' : IDL.Func([IDL.Principal], [Result_3], ['query']),
    'get_event_attendees_count' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64))],
        ['query'],
      ),
    'get_event_invites' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal],
        [Result_4],
        [],
      ),
    'get_event_invites_count' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64))],
        ['query'],
      ),
    'get_self' : IDL.Func([], [Result], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'invite_to_event' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal, IDL.Principal],
        [Result],
        [],
      ),
    'join_event' : IDL.Func([IDL.Principal, IDL.Principal], [Result], []),
    'leave_event' : IDL.Func([IDL.Principal], [Result_1], []),
    'remove_attendee_from_event' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal, IDL.Principal],
        [Result_1],
        [],
      ),
    'remove_attendee_invite_from_event' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal, IDL.Principal],
        [Result_1],
        [],
      ),
    'remove_invite' : IDL.Func([IDL.Principal], [Result_1], []),
    'restore_data' : IDL.Func([], [], []),
    'sanity_check' : IDL.Func([], [IDL.Text], ['query']),
    'total_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'upload_chunk' : IDL.Func(
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  return [IDL.Principal, IDL.Text, IDL.Nat64];
};
