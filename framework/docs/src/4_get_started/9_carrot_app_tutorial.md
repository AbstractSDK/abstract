# Recreating the carrot app from Abstract

This tutorial will walk you through the process of recreating the carrot-app

## Step by step guide

1. Go to <a href="https://github.com/AbstractSDK/app-template" target="_blank">our App Template on Github</a> and click on the "Use this template" button to create a new repository based on the template. You can name the repository whatever you want, but we recommend using the name of your module.

![](../resources/get_started/use-this-template.webp)

2. Let's call the repository `carrot-app-tutorial` and click `Create repository`
3. go to your terminal

```sh
# replace `YOUR_GITHUB_ID` with your actual github ID
git clone https://github.com/YOUR_GITHUB_ID/carrot-app-tutorial.git
cd carrot-app-tutorial
```

4.

```sh
chmod +x ./template-setup.sh
./template-setup.sh
# press (y) to install tools that we will need as we go
```

What we have now is a counter app serving as a template.
By looking at the `handlers/execute.rs` file you can see that the contract allows to incrementent or reset the counter.

```rust
match msg {
        AppExecuteMsg::Increment {} => increment(deps, app),
        AppExecuteMsg::Reset { count } => reset(deps, info, count, app),
        AppExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
```

5. Let's replace these messages by what we want to have in the carrot-app

```rust
// in handlers/execute.rs
match msg {
        // This will create a position in the supercharged liquidity pool.
        // When executing this you will add for example some liquidity for both assets USDC and USDT
        AppExecuteMsg::CreatePosition(create_position_msg) => {
            create_position(deps, env, info, app, create_position_msg)
        }
        AppExecuteMsg::Deposit { funds } => deposit(deps, env, info, funds, app),
        AppExecuteMsg::Withdraw { amount } => withdraw(deps, env, info, Some(amount), app),
        AppExecuteMsg::WithdrawAll {} => withdraw(deps, env, info, None, app),
        AppExecuteMsg::Autocompound {} => autocompound(deps, env, info, app),
    }
```

6. Now that we have defined these entrypoint msgs let's create the above mentioned functions

```rust
// in handlers/execute.rs
fn create_position(deps: DepsMut, env: Env, info: MessageInfo, app: App) -> AppResult {
    Ok(app.response("create_position"))
}
fn deposit(deps: DepsMut, env: Env, info: MessageInfo, funds: Vec<Coin>, app: App) -> AppResult {
    Ok(app.response("deposit"))
}
fn withdraw(deps: DepsMut, env: Env, info: MessageInfo, app: App) -> AppResult {
    Ok(app.response("withdraw"))
}
fn autocompound(deps: DepsMut, app: App) -> AppResult {
    Ok(app.response("autocompound"))
}
```
