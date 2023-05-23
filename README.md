# Event attendee canister

This repository is responsible for handling event members of the Catalyze application. Attendees hold the users joined events and outstanding invites.

## setup

The parent canister is SNS controlled, the child canisters are controlled by their parent. Upgrading the child canister is done through the parent canister as the (gzipped) child wasm is included in the parent canister.

When the parent canister is upgraded it checks if the child wasm has changed (currently it generates a new wasm hash every time you run the script). if changed it upgrades the child canisters automatically.

## Project structure

**|- candid**
Contains the candid files for the `parent` and `child` canister.

**|- frontend**
Contains all declarations that are needed for the frontend

**|- scripts**
Contains a single script that generates the following files for the parent and child canisters;

- candid files
- frontend declarations
- wasms (gzipped and regular)

**|- src/child**
Contains codebase related to the child canisters
**|- src/parent**
Contains codebase related to the child canisters
**|- src/shared**
Contains data used by both codebases

**|- wasm**
Contains

- child wasm
- child wasm (gzipped)
- parent wasm
- parent wasm (gzipped)

## Parent canister

The parent canister manages all underlying child canisters.

#### This canister is responsible for;

- keeping track of all event attendees child canisters
- spinning up a new child canisters
- composite query call to the children (preperation)

#### methods

Described methods can be found below, for more details you can check out the code which is inline commented

###### DEFAULT

```
// Stores the data in stable storage before upgrading the canister.
pub fn pre_upgrade() {}

// Restores the data from stable- to heap storage after upgrading the canister.
pub fn post_upgrade() {}

// Init methods thats get triggered when the canister is installed
pub fn init() {}
```

##

###### QUERY CALLS

```
// Method to retrieve an available canister to write updates to
fn get_available_canister() -> Result<ScalableCanisterDetails, String> {}

// Method to retrieve all the canisters
fn get_canisters() -> Vec<ScalableCanisterDetails> {}

// Method to retrieve the latest wasm version of the child canister that is currently stored
fn get_latest_wasm_version() -> WasmVersion {}

// HTTP request handler (canister metrics are added to the response)
fn http_request(req: HttpRequest) -> HttpResponse {}

// Method used to get all the members from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
async fn get_members(
    group_identifier: Principal,
    limit: usize,
    page: usize,
) -> PagedResponse<JoinedAttendeeResponse> {}

// Method used to get all the members from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
async fn get_invites(
    group_identifier: Principal,
    limit: usize,
    page: usize,
) -> PagedResponse<InviteAttendeeResponse> {}

```

##

###### UPDATE CALLS

```
// Method called by child canister once full (inter-canister call)
// can only be called by a child canister
async fn close_child_canister_and_spawn_sibling(
    last_entry_id: u64,
    entry: Vec<u8>
    ) -> Result<Principal, ApiError> {}

// Method to accept cycles when send to this canister
fn accept_cycles() -> u64 {}
```

## Child canister

The child canister is where the data is stored that the app uses.

This canister is responsible for;

- storing data records
- data validation
- messaging the parent to spin up a new sibling

#### methods

Described methods can be found below, for more details you can check out the code which is inline commented

###### DEFAULT

```
// Stores the data in stable storage before upgrading the canister.
pub fn pre_upgrade() {}

// Restores the data from stable- to heap storage after upgrading the canister.
pub fn post_upgrade() {}

// Init methods thats get triggered when the canister is installed
pub fn init(parent: Principal, name: String, identifier: usize) {}
```

##

###### QUERY CALLS

```
// Method to get the number of attendees for an event
fn get_event_attendees_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {}

// Method to get the number of invites for an event
fn get_event_invites_count(event_identifiers: Vec<Principal>) -> Vec<(Principal, usize)> {}

// Method to get the attendees for an event
fn get_event_attendees(
    event_identifier: Principal,
) -> Result<Vec<JoinedAttendeeResponse>, ApiError> {}

// Method to get the caller his joined events and invites
fn get_self() -> Result<(Principal, Attendee), ApiError> {}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
fn get_chunked_join_data(
    event_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get members the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
fn get_chunked_invite_data(
    event_identifier: Principal,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {}
```

###

###### UPDATE CALLS

```
// Method to join an existing event
// The method is async because it optionally creates a new canister is created
async fn join_event(
    event_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {}

// Method to invite a member to an event
async fn invite_to_event(
    event_identifier: Principal,
    attendee_principal: Principal,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {}

// Method to accept an invite to an event as a admin
async fn accept_user_request_event_invite(
    attendee_principal: Principal,
    event_identifier: Principal,
    member_identifier: Principal,
    group_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {}

// Method to accept an invite to an event as a user
async fn accept_owner_request_event_invite(
    event_identifier: Principal,
) -> Result<(Principal, Attendee), ApiError> {}

// Method to leave an event as a user
fn leave_event(event_identifier: Principal) -> Result<(), ApiError> {}

// Method to remove an event invite as a user
fn remove_invite(event_identifier: Principal) -> Result<(), ApiError> {}

// Method to remove an event attendee as a admin
async fn remove_attendee_from_event(
    attendee_principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<(), ApiError> {}

// Method to remove an event invite as a admin
async fn remove_attendee_invite_from_event(
    principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<(), ApiError> {}

// Method to get event invites for a specific event inside a group
async fn get_event_invites(
    event_identifier: Principal,
    group_identifier: Principal,
    member_identifier: Principal,
) -> Result<Vec<InviteAttendeeResponse>, ApiError> {}

// Method to add the owner as an attendee
fn add_owner_as_attendee(
    user_principal: Principal,
    event_identifier: Principal,
    group_identifier: Principal,
) -> Result<(), bool> {}
```

## SNS controlled

// TBD

## Testing

// TBD
