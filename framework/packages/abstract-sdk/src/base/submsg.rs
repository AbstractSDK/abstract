use cosmwasm_std::{Binary, CosmosMsg, DepsMut, Empty, Env, Reply, ReplyOn};

use super::response::Response;

pub type ReplyFunc<Module, Error, T> =
    fn(DepsMut, Env, &Module, Reply) -> Result<Response<Module, Error, T>, Error>;

/// A submessage that will guarantee a `reply` call on success or error, depending on
/// the `reply_on` setting. If you do not need to process the result, use regular messages instead.
///
/// Note: On error the submessage execution will revert any partial state changes due to this message,
/// but not revert any state changes in the calling contract. If this is required, it must be done
/// manually in the `reply` entry point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubMsg<Module, Error, T = Empty> {
    /// An arbitrary ID chosen by the contract.
    /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
    pub reply_func: Option<ReplyFunc<Module, Error, T>>,
    /// Some arbitrary data that the contract can set in an application specific way.
    /// This is just passed into the `reply` entry point and is not stored to state.
    /// Any encoding can be used. If `id` is used to identify a particular action,
    /// the encoding can also be different for each of those actions since you can match `id`
    /// first and then start processing the `payload`.
    ///
    /// The environment restricts the length of this field in order to avoid abuse. The limit
    /// is environment specific and can change over time. The initial default is 128 KiB.
    ///
    /// Unset/nil/null cannot be differentiated from empty data.
    ///
    /// On chains running CosmWasm 1.x this field will be ignored.
    pub payload: Binary,
    pub msg: CosmosMsg<T>,
    /// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    ///
    /// Setting this to `None` means unlimited. Then the submessage execution can consume all gas of the
    /// current execution context.
    pub gas_limit: Option<u64>,
    pub reply_on: ReplyOn,
}

impl<Module, Error, T> SubMsg<Module, Error, T> {
    /// Creates a "fire and forget" message with the pre-0.14 semantics.
    /// Since this is just an alias for [`SubMsg::reply_never`] it is somewhat recommended
    /// to use the latter in order to make the behaviour more explicit in the caller code.
    /// But that's up to you for now.
    ///
    /// By default, the submessage's gas limit will be unlimited. Use [`SubMsg::with_gas_limit`] to change it.
    /// Setting `payload` is not advised as this will never be used.
    pub fn new(msg: impl Into<CosmosMsg<T>>) -> Self {
        Self::reply_never(msg)
    }

    /// Creates a `SubMsg` that will provide a `reply` with the given `id` if the message returns `Ok`.
    ///
    /// By default, the submessage's `payload` will be empty and the gas limit will be unlimited. Use
    /// [`SubMsg::with_payload`] and [`SubMsg::with_gas_limit`] to change those.
    pub fn reply_on_success(msg: impl Into<CosmosMsg<T>>, id: ReplyFunc<Module, Error, T>) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Success)
    }

    /// Creates a `SubMsg` that will provide a `reply` with the given `id` if the message returns `Err`.
    ///
    /// By default, the submessage's `payload` will be empty and the gas limit will be unlimited. Use
    /// [`SubMsg::with_payload`] and [`SubMsg::with_gas_limit`] to change those.
    pub fn reply_on_error(msg: impl Into<CosmosMsg<T>>, id: ReplyFunc<Module, Error, T>) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Error)
    }

    /// Create a `SubMsg` that will always provide a `reply` with the given `id`.
    ///
    /// By default, the submessage's `payload` will be empty and the gas limit will be unlimited. Use
    /// [`SubMsg::with_payload`] and [`SubMsg::with_gas_limit`] to change those.
    pub fn reply_always(msg: impl Into<CosmosMsg<T>>, id: ReplyFunc<Module, Error, T>) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Always)
    }

    /// Create a `SubMsg` that will never `reply`. This is equivalent to standard message semantics.
    ///
    /// By default, the submessage's gas limit will be unlimited. Use [`SubMsg::with_gas_limit`] to change it.
    /// Setting `payload` is not advised as this will never be used.
    pub fn reply_never(msg: impl Into<CosmosMsg<T>>) -> Self {
        SubMsg {
            reply_func: None,
            payload: Default::default(),
            msg: msg.into(),
            reply_on: ReplyOn::Never,
            gas_limit: None,
        }
    }

    /// Add a gas limit to the submessage.
    /// This gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{coins, BankMsg, ReplyOn, SubMsg};
    /// # let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
    /// let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234).with_gas_limit(60_000);
    /// assert_eq!(sub_msg.id, 1234);
    /// assert_eq!(sub_msg.gas_limit, Some(60_000));
    /// assert_eq!(sub_msg.reply_on, ReplyOn::Always);
    /// ```
    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Add a payload to the submessage.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{coins, BankMsg, Binary, ReplyOn, SubMsg};
    /// # let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
    /// let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234)
    ///     .with_payload(vec![1, 2, 3, 4]);
    /// assert_eq!(sub_msg.id, 1234);
    /// assert_eq!(sub_msg.payload, Binary::new(vec![1, 2, 3, 4]));
    /// assert_eq!(sub_msg.reply_on, ReplyOn::Always);
    /// ```
    pub fn with_payload(mut self, payload: impl Into<Binary>) -> Self {
        self.payload = payload.into();
        self
    }

    fn reply_on(
        msg: CosmosMsg<T>,
        reply_func: ReplyFunc<Module, Error, T>,
        reply_on: ReplyOn,
    ) -> Self {
        SubMsg {
            reply_func: Some(reply_func),
            payload: Default::default(),
            msg,
            reply_on,
            gas_limit: None,
        }
    }
}
