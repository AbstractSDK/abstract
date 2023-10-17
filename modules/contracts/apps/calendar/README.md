# Calendar App module

The Calendar app module is used to require people to stake-to-schedule time with you to avoid not showing up.

## Features
- Allow admin to specify
    - How much they charge per minute to determine the stake of a meeting
    - The denom of the staked asset
    - The utc offset to determine the preferred timezone
    - The start and end times to determine the valid bounds for a given meeting
- Allow anyone to schedule a meeting by specifying the start and end times of the meeting along with the required stake. These times are specified as unix timestamps using an 64 bit signed integer. This datatype follows the unix time
spec that timestamps should be able to be negative (to go back in time) and the 64 bits ensure that this module is compliant with the 2038 problem faced by using a signed 32 bit integer.
- Admin can manage the stakes in a few ways:
    - Return stake: If the requester attended the meeting as planned their entire stake is returned.
    - Slash partial stake: If the requester was late but still attended part of their stake gets slashed based on how late they were.
    - Slash full stake: If the requester never showed up the entire stake gets slashed.

## Installation
To use the Calendar App Module in your Rust project, add the following dependency to your `Cargo.toml`:
```toml
[dependencies]
calendar-app = { git = "https://github.com/AbstractSDK/abstract.git", tag="v0.19.0", default-features = false }
```

## Usage with the Abstract SDK
To interact with the calendar, you first need to retrieve the calendar using the Calendar App. Here's a basic example in Rust:
```rust
// Retrieve the calendar
use calendar_app::AppInterface;
...

let calendar = app.calendar(deps.as_ref());
let meeting_res = calendar.request_meeting(end_time, start_time)?;
```
