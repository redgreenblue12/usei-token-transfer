#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, Addr, BankMsg, CosmosMsg, Deps, DepsMut};
    use cosmwasm_storage::Bucket;

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, GetOwnerResponse, GetWithdrawableCoinQuantityResponse, InstantiateMsg, QueryMsg,
    };
    use crate::state::{resolver, AccountBalance, Config, Fee};
    use crate::ContractError;

    fn assert_config_state(deps: Deps, expected: Config) {
        let res = query(deps, mock_env(), QueryMsg::Config {}).unwrap();
        let value: Config = from_binary(&res).unwrap();
        assert_eq!(value, expected);
    }

    fn set_address_with_coins(
        resolver: &mut Bucket<AccountBalance>,
        address: &str,
        coin_quantity: u128,
    ) {
        let new_account_balance = AccountBalance {
            address: Addr::unchecked(address),
            balance: coin_quantity,
        };
        resolver.save(address.as_bytes(), &new_account_balance).ok();
    }

    fn assert_account_balance(deps: Deps, address: &str, expected_balance: u128) {
        let res = query(
            deps,
            mock_env(),
            QueryMsg::GetWithdrawableCoinQuantity {
                address: address.to_string(),
            },
        )
        .unwrap();
        let data: GetWithdrawableCoinQuantityResponse = from_binary(&res).unwrap();

        assert_eq!(data.address, address);
        assert_eq!(data.balance, expected_balance);
    }

    fn mock_init_no_owner_specified(deps: DepsMut) {
        let msg = InstantiateMsg {
            owner: None,
            flat_fee: None,
            percent_fee: None,
        };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    fn mock_init_with_fees(deps: DepsMut, flat_fee: u128, percent_fee: u128) {
        let msg = InstantiateMsg {
            owner: None,
            flat_fee: Some(flat_fee),
            percent_fee: Some(percent_fee),
        };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    fn mock_init_owner_specified(deps: DepsMut, owner: String) {
        let msg = InstantiateMsg {
            owner: Some(owner),
            flat_fee: None,
            percent_fee: None,
        };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    #[test]
    fn create_contract_with_implict_owner() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());
        assert_config_state(
            deps.as_ref(),
            Config {
                owner: Addr::unchecked("creator"),
                fee: Fee {
                    flat_fee: 0,
                    percent_fee: 0,
                },
            },
        )
    }

    #[test]
    fn create_contract_with_explicit_owner() {
        let mut deps = mock_dependencies();
        mock_init_owner_specified(deps.as_mut(), String::from("someone"));
        assert_config_state(
            deps.as_ref(),
            Config {
                owner: Addr::unchecked("someone"),
                fee: Fee {
                    flat_fee: 0,
                    percent_fee: 0,
                },
            },
        )
    }

    #[test]
    fn create_contract_with_bad_percent() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: None,
            flat_fee: Some(0),
            percent_fee: Some(10000000),
        };

        let info = mock_info("creator", &coins(2, "token"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ContractError::PercentFeeTooLarge { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn query_contract_owner() {
        let mut deps = mock_dependencies();
        mock_init_owner_specified(deps.as_mut(), String::from("someone"));

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let data: GetOwnerResponse = from_binary(&res).unwrap();

        assert_eq!(data.owner, "someone");
    }

    #[test]
    fn query_account_balance_info() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        // Sanity check that Alice's account balance is initially empty.
        assert_account_balance(deps.as_ref(), "alice", 0);

        // Manually set Alice's account balance to 123 and expect it in the subsequent query.
        set_address_with_coins(&mut resolver(&mut deps.storage), "alice", 123);

        assert_account_balance(deps.as_ref(), "alice", 123);
    }

    #[test]
    fn send_coins_to_two_new_accounts_even_split() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        let info = mock_info("creator", &[coin(300, "usei")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins");

        // Alice should have exactly half the coins
        // Bob should have the other half of the coins
        // Joe should still have no coins
        assert_account_balance(deps.as_ref(), "alice", 150);
        assert_account_balance(deps.as_ref(), "bob", 150);
        assert_account_balance(deps.as_ref(), "joe", 0);
    }

    #[test]
    fn send_coins_with_flat_fees() {
        let mut deps = mock_dependencies();
        mock_init_with_fees(deps.as_mut(), 20, 0);

        let info = mock_info("creator", &[coin(300, "usei")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins");

        // 300 coins were sent, minus the flat fee (20), so Alice and Bob should have
        // 140 coins each.
        assert_account_balance(deps.as_ref(), "alice", 140);
        assert_account_balance(deps.as_ref(), "bob", 140);
        // The contract owner, 'creator', should have 20 coins now.
        assert_account_balance(deps.as_ref(), "creator", 20);
    }

    #[test]
    fn send_coins_with_percent_fees() {
        let mut deps = mock_dependencies();
        mock_init_with_fees(deps.as_mut(), 0, 5000);

        let info = mock_info("creator", &[coin(300, "usei")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins");

        // 300 coins were sent, minus the percent fee (50%), so Alice and Bob should have
        // 75 coins each.
        assert_account_balance(deps.as_ref(), "alice", 75);
        assert_account_balance(deps.as_ref(), "bob", 75);
        // The contract owner, 'creator', should have 150 coins now.
        assert_account_balance(deps.as_ref(), "creator", 150);
    }

    #[test]
    fn send_coins_to_two_new_accounts_odd_split() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        let info = mock_info("creator", &[coin(15, "usei")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins");

        // Alice and Bob should both have their half of the coins, rounded down.
        // No errors should be thrown.
        assert_account_balance(deps.as_ref(), "alice", 7);
        assert_account_balance(deps.as_ref(), "bob", 7);
    }

    #[test]
    fn send_coins_to_two_existing_accounts() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        let info = mock_info("creator", &[coin(20, "usei")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins the first time");

        let info = mock_info("creator", &[coin(30, "usei")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins the second time");

        // Between the two transactions, 50 coins were sent.
        // Alice and Bob should both have 25 coins each.
        assert_account_balance(deps.as_ref(), "alice", 25);
        assert_account_balance(deps.as_ref(), "bob", 25);
    }

    #[test]
    fn send_unrelated_coin() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        // Instead of sending 'usei', send 'ueth'.
        let info = mock_info("creator", &[coin(20, "ueth")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins");

        // Alice and Bob should *not* see their balances increase.
        assert_account_balance(deps.as_ref(), "alice", 0);
        assert_account_balance(deps.as_ref(), "bob", 0);
    }

    #[test]
    fn send_usei_and_unrelated_coins_together() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        // Send both 'usei' and 'ueth'.
        let info = mock_info("creator", &[coin(10, "usei"), coin(20, "ueth")]);
        let msg = ExecuteMsg::SendCoins {
            dest_addr1: String::from("alice"),
            dest_addr2: String::from("bob"),
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully sent the coins");

        // Alice and Bob should *not* see their balances increase by 5 each, since 10 USEI was sent total.
        assert_account_balance(deps.as_ref(), "alice", 5);
        assert_account_balance(deps.as_ref(), "bob", 5);
    }

    #[test]
    fn withdraw_coins() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        // Give Alice 5 coins to start off with.
        set_address_with_coins(&mut resolver(&mut deps.storage), "alice", 5);
        assert_account_balance(deps.as_ref(), "alice", 5);

        // After withdrawing 2 coins, Alice should now only have 3.
        let info = mock_info("alice", &[]);
        let msg = ExecuteMsg::WithdrawCoins { quantity: 2 };

        let res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully created the withdraw event");

        // Make sure the message is sent for wasm to chain.
        let msg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "alice".into(),
                amount: coins(2, "usei"),
            })
        );

        assert_account_balance(deps.as_ref(), "alice", 3);

        // Aice can withdraw her remaining 3 coins, so she is left with 0.
        let info = mock_info("alice", &[]);
        let msg = ExecuteMsg::WithdrawCoins { quantity: 3 };

        let _res = execute(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully created the withdraw event");

        assert_account_balance(deps.as_ref(), "alice", 0);
    }

    #[test]
    fn withdraw_coins_not_enough_funds() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        // Give Alice 5 coins to start off with.
        set_address_with_coins(&mut resolver(&mut deps.storage), "alice", 5);
        assert_account_balance(deps.as_ref(), "alice", 5);

        // Alice cannot withdraw 10 coins, since she only has 5.
        let info = mock_info("alice", &[]);
        let msg = ExecuteMsg::WithdrawCoins { quantity: 10 };

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ContractError::InsufficientFunds { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }

        // Alice should still have her 5 coins left.
        assert_account_balance(deps.as_ref(), "alice", 5);
    }

    #[test]
    fn withdraw_coins_empty_quantity() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());

        // Give Alice 5 coins to start off with.
        set_address_with_coins(&mut resolver(&mut deps.storage), "alice", 5);
        assert_account_balance(deps.as_ref(), "alice", 5);

        // Alice cannot withdraw 10 coins, since she only has 5.
        let info = mock_info("alice", &[]);
        let msg = ExecuteMsg::WithdrawCoins { quantity: 0 };

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ContractError::EmptyWithdrawQuantity { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
