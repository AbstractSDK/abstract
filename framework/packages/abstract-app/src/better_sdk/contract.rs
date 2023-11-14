



    // /// Handler of instantiate messages.
    // pub(crate) instantiate_handler:
    //     Option<InstantiateHandlerFn<Module, <Module as Handler>::CustomInitMsg, Error>>,
    // /// Handler of execute messages.
    // pub(crate) execute_handler:
    //     Option<ExecuteHandlerFn<Module, <Module as Handler>::CustomExecMsg, Error>>,
    // /// Handler of query messages.
    // pub(crate) query_handler:
    //     Option<QueryHandlerFn<Module, <Module as Handler>::CustomQueryMsg, Error>>,
    // /// Handler for migrations.
    // pub(crate) migrate_handler:
    //     Option<MigrateHandlerFn<Module, <Module as Handler>::CustomMigrateMsg, Error>>,
    // /// Handler for sudo messages.
    // pub(crate) sudo_handler: Option<SudoHandlerFn<Module, <Module as Handler>::SudoMsg, Error>>,
    // /// List of reply handlers per reply ID.
    // pub reply_handlers: [&'static [(u64, ReplyHandlerFn<Module, Error>)]; MAX_REPLY_COUNT],
    // /// Handler of `Receive variant Execute messages.
    // pub(crate) receive_handler:
    //     Option<ReceiveHandlerFn<Module, <Module as Handler>::ReceiveMsg, Error>>,
    // /// IBC callbacks handlers following an IBC action, per callback ID.
    // pub(crate) ibc_callback_handlers:
    //     &'static [(&'static str, IbcCallbackHandlerFn<Module, Error>)],