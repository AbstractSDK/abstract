# Calendar App module

## Description

The Calendar app module is used to require people to stake-to-schedule time with you to avoid not showing up.

## Why use the Calendar App?

The Calendar App offers several potential benefits, particularly in enhancing the efficiency and respect for one's time in professional and personal scheduling. Here are some key advantages:

1. **Reduction in No-shows**: The primary benefit of requiring a stake to schedule time is to significantly reduce the likelihood of no-shows. When there is a financial or valuable asset at stake, individuals are more likely to commit to the meeting or appointment, knowing that failure to show up would result in a loss.

2. **Prioritization of Meetings**: This system ensures that only serious and important meetings are scheduled, as the stake acts as a filter to prioritize requests. It dissuades casual or non-essential meetings, thus helping to manage one's calendar more effectively.

3. **Enhanced Time Management**: By reducing the number of frivolous meetings and no-shows, individuals can manage their time more effectively. This leads to increased productivity as the time saved can be allocated to other pressing tasks or personal downtime.

4. **Incentivizes Punctuality and Preparation**: Knowing that there is something at stake, participants are more likely to be punctual and come prepared to the meeting. This can improve the overall quality and efficiency of the meeting.

5. **Possible Revenue Generation**: For professionals whose time is highly valuable, this system could also serve as a means of revenue generation. The stakes collected from no-shows or cancellations could potentially be a source of income or be used to cover the administrative costs of scheduling and organizing meetings.

6. **Enhances Respect for Individual's Time**: Implementing such a system sends a strong message about the value of one's time. It fosters a culture of respect and consideration for others' schedules, which is beneficial in both professional and personal interactions.

7. **Customization and Flexibility**: The app could offer customizable options for the stake amount, which can be adjusted based on the nature of the meeting, the relationship between the parties, or the professional standing of the individual. This flexibility can make it suitable for various contexts and preferences.

8. **Automated Scheduling and Enforcement**: Utilizing a digital platform for this process can streamline scheduling, reminders, and enforcement of the stake policy. Automation reduces the administrative burden and ensures a smooth, efficient process.

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
calendar-app = { git = "https://github.com/AbstractSDK/abstract.git", tag="<latest-tag>", default-features = false }
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

## Documentation

- **App Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/6_module_types.html#apps).

## Contributing

If you have suggestions, improvements or want to contribute to the project, we welcome your input on [GitHub](https://github.com/AbstractSDK/abstract).

## Community
Check out the following places for support, discussions & feedback:

- Join our [Discord server](https://discord.com/invite/uch3Tq3aym)
