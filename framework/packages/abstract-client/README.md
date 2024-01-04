# Abstract Client

Abstract client allows you to do everything you might need to work with the Abstract
or to be more precise

- Create or interact with Account
- Install or interact with a module (including apps and adapters)
- Publish modules
- Do integration tests with Abstract

Example of publishing mock app

```rust
use abstract_app::mock::interface::MockAppInterface;
use cw_orch::prelude::Mock;
use abstract_client::{client::AbstractClient, publisher::Publisher};
let client = AbstractClient::builder("sender").build()?;
let namespace = "tester";
let publisher: Publisher<Mock> = client
    .publisher_builder(namespace)
    .build()?;
publisher.publish_app::<MockAppInterface<Mock>>()?;
```
