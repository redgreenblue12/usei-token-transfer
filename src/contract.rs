#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cosmwasm_storage::Bucket;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetOwnerResponse, GetWithdrawableCoinQuantityResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{config, config_read, resolver, resolver_read, AccountBalance, Config};

pub static COIN_DENOM: &str = "usei";

/*
const CONTRACT_NAME: &str = "crates.io:usei-transfer-tokens";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
 */

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Get the owner either from the message that the creator instantiates with, if they
    // specify an owner.
    // Alternatively, set the owner as the creator.
    let owner: Result<Addr, StdError> = match msg.owner {
        Some(explicit_owner) => Ok(deps.api.addr_validate(&explicit_owner)?),
        None => Ok(info.sender),
    };

    // Instantiate the contract.
    let config_state: Config = Config { owner: owner? };
    config(deps.storage).save(&config_state)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SendCoins {
            dest_addr1,
            dest_addr2,
        } => execute_send_coins(deps, env, info, dest_addr1, dest_addr2),
        ExecuteMsg::WithdrawCoins { quantity } => execute_withdraw_coins(deps, env, info, quantity),
    }
}

fn execute_send_coins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    dest_addr1: String,
    dest_addr2: String,
) -> Result<Response, ContractError> {
    let valid_dest_addr1 = deps.api.addr_validate(&dest_addr1)?;
    let valid_dest_addr2 = deps.api.addr_validate(&dest_addr2)?;
    let total_coin_quantity = get_coin_quantity_sent_in_message(info);
    let split_coin_quantity = total_coin_quantity / 2;
    increase_coins_at_address(
        &mut resolver(deps.storage),
        valid_dest_addr1,
        split_coin_quantity,
    )?;
    increase_coins_at_address(
        &mut resolver(deps.storage),
        valid_dest_addr2,
        split_coin_quantity,
    )?;
    Ok(Response::default())
}

fn increase_coins_at_address(
    resolver: &mut Bucket<AccountBalance>,
    valid_dest_addr: Addr,
    coin_quantity: u128,
) -> Result<AccountBalance, ContractError> {
    let key = valid_dest_addr.as_bytes();
    resolver.update(key, |account_balance: Option<AccountBalance>| {
        if let Some(mut account_balance) = account_balance {
            account_balance.balance = account_balance.balance + coin_quantity;
            Ok::<AccountBalance, ContractError>(account_balance)
        } else {
            let new_balance = AccountBalance {
                address: valid_dest_addr.clone(),
                balance: coin_quantity,
            };
            Ok(new_balance)
        }
    })
}

fn execute_withdraw_coins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    quantity: u128,
) -> Result<Response, ContractError> {
    let address = info.sender;
    if quantity == 0 {
        return Err(ContractError::EmptyWithdrawQuantity {
            withdraw_quantity: quantity,
        });
    };
    decrease_coins_at_address(&mut resolver(deps.storage), &address, quantity)?;
    let resp = Response::new()
        .add_message(BankMsg::Send {
            to_address: address.clone().into(),
            amount: vec![Coin {
                denom: COIN_DENOM.to_string(),
                amount: Uint128::from(quantity),
            }],
        })
        .add_attribute("action", "withdraw")
        .add_attribute("to", address);
    Ok(resp)
}

fn decrease_coins_at_address(
    resolver: &mut Bucket<AccountBalance>,
    valid_dest_addr: &Addr,
    coin_quantity: u128,
) -> Result<AccountBalance, ContractError> {
    let key = valid_dest_addr.as_bytes();
    resolver.update(key, |account_balance: Option<AccountBalance>| {
        if let Some(mut account_balance) = account_balance {
            if account_balance.balance >= coin_quantity {
                account_balance.balance = account_balance.balance - coin_quantity;
                Ok::<AccountBalance, ContractError>(account_balance)
            } else {
                Err(ContractError::InsufficientFunds {
                    withdraw_quantity: coin_quantity,
                    balance: account_balance.balance,
                })
            }
        } else {
            Err(ContractError::InsufficientFunds {
                withdraw_quantity: coin_quantity,
                balance: 0,
            })
        }
    })
}

fn get_coin_quantity_sent_in_message(info: MessageInfo) -> u128 {
    let coins_sent = info.funds.iter().find(|coin| coin.denom == COIN_DENOM);
    match coins_sent {
        Some(coins) => coins.amount.u128(),
        None => 0,
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => query_get_owner(deps, env, msg),
        QueryMsg::Config {} => query_config(deps, env, msg),
        QueryMsg::GetWithdrawableCoinQuantity { address } => {
            query_get_account_balance(deps, env, address)
        }
    }
}

fn query_get_owner(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let config_data = config_read(deps.storage).load()?;
    let resp = GetOwnerResponse {
        owner: String::from(config_data.owner),
    };
    to_binary(&resp)
}

fn query_config(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let config_data = config_read(deps.storage).load()?;
    to_binary(&config_data)
}

fn query_get_account_balance(deps: Deps, _env: Env, address: String) -> StdResult<Binary> {
    let valid_address = deps.api.addr_validate(&address)?;
    let balance = get_account_balance(deps, &valid_address)?;
    let resp = GetWithdrawableCoinQuantityResponse { address, balance };
    to_binary(&resp)
}

fn get_account_balance(deps: Deps, valid_address: &Addr) -> Result<u128, StdError> {
    let key = valid_address.as_bytes();
    let balance = match resolver_read(deps.storage).may_load(key)? {
        Some(ab) => ab.balance,
        None => 0,
    };
    Ok(balance)
}
