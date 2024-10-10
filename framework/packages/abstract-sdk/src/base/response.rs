use std::marker::PhantomData;

use cosmwasm_std::Binary;

use cosmwasm_std::{Attribute, CosmosMsg, Empty, Event};

use crate::AbstractSdkError;

use super::submsg::SubMsg;
use super::{AbstractContract, Handler};
/// A response of a contract entry point, such as `instantiate`, `execute` or `migrate`.
///
/// This type can be constructed directly at the end of the call. Alternatively a
/// mutable response instance can be created early in the contract's logic and
/// incrementally be updated.
///
/// ## Examples
///
/// Direct:
///
/// ```
/// # use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo};
/// # type InstantiateMsg = ();
/// #
/// use cosmwasm_std::{attr, Response, StdResult};
///
/// pub fn instantiate(
///     deps: DepsMut,
///     _env: Env,
///     _info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> StdResult<Response> {
///     // ...
///
///     Ok(Response::new().add_attribute("action", "instantiate"))
/// }
/// ```
///
/// Mutating:
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, Binary, DepsMut, Env, MessageInfo, SubMsg};
/// # type InstantiateMsg = ();
/// # type MyError = ();
/// #
/// use cosmwasm_std::Response;
///
/// pub fn instantiate(
///     deps: DepsMut,
///     _env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, MyError> {
///     let mut response = Response::new()
///         .add_attribute("Let the", "hacking begin")
///         .add_message(BankMsg::Send {
///             to_address: String::from("recipient"),
///             amount: coins(128, "uint"),
///         })
///         .add_attribute("foo", "bar")
///         .set_data(b"the result data");
///     Ok(response)
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Response<Module, Error, T = Empty> {
    /// Optional list of messages to pass. These will be executed in order.
    /// If the ReplyOn variant matches the result (Always, Success on Ok, Error on Err),
    /// the runtime will invoke this contract's `reply` entry point
    /// after execution. Otherwise, they act like "fire and forget".
    /// Use `SubMsg::new` to create messages with the older "fire and forget" semantics.
    pub messages: Vec<SubMsg<Module, Error, T>>,
    /// The attributes that will be emitted as part of a "wasm" event.
    ///
    /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/learn/advanced/events
    pub attributes: Vec<Attribute>,
    /// Extra, custom events separate from the main `wasm` one. These will have
    /// `wasm-` prepended to the type.
    ///
    /// More info about events can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/learn/advanced/events
    pub events: Vec<Event>,
    /// The binary payload to include in the response.
    pub data: Option<Binary>,

    pub ph: PhantomData<Module>,
}

impl<Module, Error, T> Default for Response<Module, Error, T> {
    fn default() -> Self {
        Response {
            messages: vec![],
            attributes: vec![],
            events: vec![],
            data: None,
            ph: Default::default(),
        }
    }
}

impl<Module, Error, T> Response<Module, Error, T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attribute included in the main `wasm` event.
    ///
    /// For working with optional values or optional attributes, see [`add_attributes`][Self::add_attributes].
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute::new(key, value));
        self
    }

    /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
    /// and adds it to the list of messages to process.
    pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
        self.messages.push(SubMsg::new(msg));
        self
    }

    /// This takes an explicit SubMsg (creates via eg. `reply_on_error`)
    /// and adds it to the list of messages to process.
    pub fn add_submessage(mut self, msg: SubMsg<Module, Error, T>) -> Self {
        self.messages.push(msg);
        self
    }

    /// Adds an extra event to the response, separate from the main `wasm` event
    /// that is always created.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided type
    /// of event.
    pub fn add_event(mut self, event: impl Into<Event>) -> Self {
        self.events.push(event.into());
        self
    }

    /// Bulk add attributes included in the main `wasm` event.
    ///
    /// Anything that can be turned into an iterator and yields something
    /// that can be converted into an `Attribute` is accepted.
    ///
    /// ## Examples
    ///
    /// Adding a list of attributes using the pair notation for key and value:
    ///
    /// ```
    /// use cosmwasm_std::Response;
    ///
    /// let attrs = vec![
    ///     ("action", "reaction"),
    ///     ("answer", "42"),
    ///     ("another", "attribute"),
    /// ];
    /// let res: Response = Response::new().add_attributes(attrs.clone());
    /// assert_eq!(res.attributes, attrs);
    /// ```
    ///
    /// Adding an optional value as an optional attribute by turning it into a list of 0 or 1 elements:
    ///
    /// ```
    /// use cosmwasm_std::{Attribute, Response};
    ///
    /// // Some value
    /// let value: Option<String> = Some("sarah".to_string());
    /// let attribute: Option<Attribute> = value.map(|v| Attribute::new("winner", v));
    /// let res: Response = Response::new().add_attributes(attribute);
    /// assert_eq!(res.attributes, [Attribute {
    ///     key: "winner".to_string(),
    ///     value: "sarah".to_string(),
    /// }]);
    ///
    /// // No value
    /// let value: Option<String> = None;
    /// let attribute: Option<Attribute> = value.map(|v| Attribute::new("winner", v));
    /// let res: Response = Response::new().add_attributes(attribute);
    /// assert_eq!(res.attributes.len(), 0);
    /// ```
    pub fn add_attributes<A: Into<Attribute>>(
        mut self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self {
        self.attributes.extend(attrs.into_iter().map(A::into));
        self
    }

    /// Bulk add "fire and forget" messages to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{CosmosMsg, Response};
    ///
    /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> Response {
    ///     Response::new().add_messages(msgs)
    /// }
    /// ```
    pub fn add_messages<M: Into<CosmosMsg<T>>>(self, msgs: impl IntoIterator<Item = M>) -> Self {
        self.add_submessages(msgs.into_iter().map(SubMsg::new))
    }

    /// Bulk add explicit SubMsg structs to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{SubMsg, Response};
    ///
    /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> Response {
    ///     Response::new().add_submessages(msgs)
    /// }
    /// ```
    pub fn add_submessages(
        mut self,
        msgs: impl IntoIterator<Item = SubMsg<Module, Error, T>>,
    ) -> Self {
        self.messages.extend(msgs);
        self
    }

    /// Bulk add custom events to the response. These are separate from the main
    /// `wasm` event.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided types
    /// of events.
    pub fn add_events<E>(mut self, events: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<Event>,
    {
        self.events.extend(events.into_iter().map(|e| e.into()));
        self
    }

    /// Set the binary data included in the response.
    pub fn set_data(mut self, data: impl Into<Binary>) -> Self {
        self.data = Some(data.into());
        self
    }
}

impl<Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> Response<Module, Error> {
    pub fn into_cosmwasm_response(
        self,
        contract: &AbstractContract<Module, Error>,
    ) -> cosmwasm_std::Response {
        let mut response = cosmwasm_std::Response::new();

        if let Some(data) = self.data {
            response = response.set_data(data)
        }
        response
            .add_attributes(self.attributes)
            .add_events(self.events)
            .add_submessages(self.messages.into_iter().map(|m| {
                const UNUSED_MSG_ID: usize = 0;
                let id = m
                    .reply_func
                    .and_then(|func| {
                        contract.reply_handlers[0]
                            .iter()
                            .position(|reply| reply == &func)
                    })
                    .unwrap_or(UNUSED_MSG_ID) as u64;
                cosmwasm_std::SubMsg {
                    id,
                    payload: m.payload,
                    msg: m.msg,
                    gas_limit: m.gas_limit,
                    reply_on: m.reply_on,
                }
            }))
    }
}

impl<Module, Error> From<cosmwasm_std::Response> for Response<Module, Error> {
    fn from(value: cosmwasm_std::Response) -> Self {
        let mut response = Response::new();
        if let Some(data) = value.data {
            response = response.set_data(data)
        }
        response
            .add_submessages(value.messages.into_iter().map(|m| SubMsg {
                reply_func: None,
                payload: m.payload,
                msg: m.msg,
                gas_limit: m.gas_limit,
                reply_on: cosmwasm_std::ReplyOn::Never,
            }))
            .add_attributes(value.attributes)
            .add_events(value.events)
    }
}
