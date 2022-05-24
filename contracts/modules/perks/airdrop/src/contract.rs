use abstract_os::tokenomics::airdrop::{
    ClaimResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
    UserInfoResponse,
};
use abstract_os::tokenomics::helpers::build_transfer_cw20_token_msg;
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};

use crate::state::{Config, State, CONFIG, STATE, USERS};
use sha3::{Digest, Keccak256};
use std::cmp::Ordering;
use std::convert::TryInto;

//----------------------------------------------------------------------------------------
// Entry points
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let from_timestamp = msg
        .from_timestamp
        .unwrap_or_else(|| env.block.time.seconds());

    if msg.to_timestamp <= from_timestamp {
        return Err(StdError::generic_err(
            "Invalid airdrop claim window closure timestamp",
        ));
    }

    let owner = if let Some(owner) = msg.owner {
        deps.api.addr_validate(&owner)?
    } else {
        info.sender
    };

    if msg.total_airdrop_size.is_zero() {
        return Err(StdError::generic_err("Invalid total airdrop amount"));
    }

    let config = Config {
        owner,
        whale_token_address: deps.api.addr_validate(&msg.whale_token_address)?,
        merkle_roots: msg.merkle_roots.unwrap_or_default(),
        from_timestamp,
        to_timestamp: msg.to_timestamp,
    };

    let state = State {
        total_airdrop_size: msg.total_airdrop_size,
        unclaimed_tokens: msg.total_airdrop_size,
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            merkle_roots,
            from_timestamp,
            to_timestamp,
        } => handle_update_config(
            deps,
            info,
            owner,
            merkle_roots,
            from_timestamp,
            to_timestamp,
        ),
        ExecuteMsg::Claim {
            claim_amount,
            merkle_proof,
            root_index,
        } => handle_claim(deps, env, info, claim_amount, merkle_proof, root_index),
        ExecuteMsg::TransferUnclaimedTokens { recepient, amount } => {
            handle_transfer_unclaimed_tokens(deps, env, info, recepient, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::HasUserClaimed { address } => to_binary(&query_user_claimed(deps, address)?),
        QueryMsg::UserInfo { address } => to_binary(&query_user_info(deps, address)?),
    }
}

//----------------------------------------------------------------------------------------
// Handle functions
//----------------------------------------------------------------------------------------

/// @dev Admin function to update Configuration parameters
/// @param new_config : Same as InstantiateMsg struct
pub fn handle_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    merkle_roots: Option<Vec<String>>,
    from_timestamp: Option<u64>,
    to_timestamp: Option<u64>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: ONLY OWNER CAN CALL THIS FUNCTION
    if info.sender != config.owner {
        return Err(StdError::generic_err("Only owner can update configuration"));
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(merkle_roots) = merkle_roots {
        config.merkle_roots = merkle_roots
    }

    if let Some(from_timestamp) = from_timestamp {
        config.from_timestamp = from_timestamp
    }

    if let Some(to_timestamp) = to_timestamp {
        if to_timestamp <= config.from_timestamp {
            return Err(StdError::generic_err(
                "Invalid airdrop claim window closure timestamp",
            ));
        }

        config.to_timestamp = to_timestamp
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "UpdateConfig"))
}

/// @dev Executes an airdrop claim for a Terra User
/// @param claim_amount : Airdrop to be claimed by the user
/// @param merkle_proof : Array of hashes to prove the input is a leaf of the Merkle Tree
/// @param root_index : Merkle Tree root identifier to be used for verification
pub fn handle_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    claim_amount: Uint128,
    merkle_proof: Vec<String>,
    root_index: u32,
) -> Result<Response, StdError> {
    let recipient = info.sender;

    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // CHECK :: IS AIRDROP CLAIM WINDOW OPEN ?
    if config.from_timestamp > env.block.time.seconds() {
        return Err(StdError::generic_err("Claim not allowed"));
    }

    // CHECK :: IS AIRDROP CLAIM WINDOW OPEN ?
    if config.to_timestamp < env.block.time.seconds() {
        return Err(StdError::generic_err("Claim period has concluded"));
    }

    let merkle_root = config.merkle_roots.get(root_index as usize);
    if merkle_root.is_none() {
        return Err(StdError::generic_err("Incorrect Merkle Root Index"));
    }

    if !verify_claim(&recipient, claim_amount, merkle_proof, merkle_root.unwrap()) {
        return Err(StdError::generic_err("Incorrect Merkle Proof"));
    }

    let mut user_info = USERS.load(deps.storage, &recipient).unwrap_or_default();

    // Check if addr has already claimed the tokens
    if !user_info.claimed_amount.is_zero() {
        return Err(StdError::generic_err("Already claimed"));
    }

    // Update amounts
    state.unclaimed_tokens -= claim_amount;
    user_info.claimed_amount = claim_amount;
    user_info.timestamp = env.block.time.seconds();

    // TRANSFER WHALE IF CLAIMS ARE ALLOWED (i.e LP Boostrap auction has concluded)
    let messages = vec![build_transfer_cw20_token_msg(
        recipient.clone(),
        config.whale_token_address.to_string(),
        user_info.claimed_amount,
    )?];

    USERS.save(deps.storage, &recipient, &user_info)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "Claim"),
        attr("addr", recipient),
        attr("airdrop", claim_amount),
    ]))
}

/// @dev Admin function to transfer WHALE Tokens to the recepient address
/// @param recepient Recepient receiving the WHALE tokens
/// @param amount Amount of WHALE to be transferred
pub fn handle_transfer_unclaimed_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // CHECK :: CAN ONLY BE CALLED BY THE OWNER
    if info.sender != config.owner {
        return Err(StdError::generic_err("Sender not authorized!"));
    }

    // CHECK :: CAN ONLY BE CALLED AFTER THE CLAIM PERIOD IS OVER
    if config.to_timestamp > _env.block.time.seconds() {
        return Err(StdError::generic_err(format!(
            "{} seconds left before unclaimed tokens can be transferred",
            { config.to_timestamp - _env.block.time.seconds() }
        )));
    }

    // CHECK :: Amount needs to be less than unclaimed_tokens balance
    if amount > state.unclaimed_tokens {
        return Err(StdError::generic_err(
            "Amount cannot exceed unclaimed token balance",
        ));
    }

    // COSMOS MSG :: TRANSFER WHALE TOKENS
    state.unclaimed_tokens -= amount;
    let transfer_msg = build_transfer_cw20_token_msg(
        deps.api.addr_validate(&recipient)?,
        config.whale_token_address.to_string(),
        amount,
    )?;

    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attributes(vec![
            attr("action", "TransferUnclaimedRewards"),
            attr("recipient", recipient),
            attr("amount", amount),
        ]))
}

//----------------------------------------------------------------------------------------
// Query functions
//----------------------------------------------------------------------------------------

/// @dev Returns the airdrop configuration
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        whale_token_address: config.whale_token_address.to_string(),
        owner: config.owner.to_string(),
        merkle_roots: config.merkle_roots,
        from_timestamp: config.from_timestamp,
        to_timestamp: config.to_timestamp,
    })
}

/// @dev Returns the airdrop contract state
fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        total_airdrop_size: state.total_airdrop_size,
        unclaimed_tokens: state.unclaimed_tokens,
    })
}

/// @dev Returns details around user's WHALE Airdrop claim
fn query_user_info(deps: Deps, user_address: String) -> StdResult<UserInfoResponse> {
    let user_address = deps.api.addr_validate(&user_address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    Ok(UserInfoResponse {
        airdrop_amount: user_info.claimed_amount,
        timestamp: user_info.timestamp,
    })
}

/// @dev Returns true if the user has claimed the airdrop [EVM addresses to be provided in lower-case without the '0x' prefix]
fn query_user_claimed(deps: Deps, address: String) -> StdResult<ClaimResponse> {
    let user_address = deps.api.addr_validate(&address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();

    Ok(ClaimResponse {
        is_claimed: !user_info.claimed_amount.is_zero(),
    })
}

//----------------------------------------------------------------------------------------
// Helper functions
//----------------------------------------------------------------------------------------

/// @dev Verify whether a claim is valid
/// @param account Account on behalf of which the airdrop is to be claimed (etherum addresses without `0x` prefix)
/// @param amount Airdrop amount to be claimed by the user
/// @param merkle_proof Array of hashes to prove the input is a leaf of the Merkle Tree
/// @param merkle_root Hash of Merkle tree's root
pub fn verify_claim(
    account: &Addr,
    amount: Uint128,
    merkle_proof: Vec<String>,
    merkle_root: &str,
) -> bool {
    let leaf = account.to_string() + &amount.to_string();
    let mut hash_buf = Keccak256::digest(leaf.as_bytes())
        .as_slice()
        .try_into()
        .expect("Wrong length");
    let mut hash_str: String;

    for p in merkle_proof {
        let mut proof_buf: [u8; 32] = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf).unwrap();
        let proof_buf_str = hex::encode(proof_buf);
        hash_str = hex::encode(hash_buf);

        if proof_buf_str.cmp(&hash_str.clone()) == Ordering::Greater {
            hash_buf = Keccak256::digest(&[hash_buf, proof_buf].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        } else {
            hash_buf = Keccak256::digest(&[proof_buf, hash_buf].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        }
    }

    hash_str = hex::encode(hash_buf);
    merkle_root == hash_str
}
