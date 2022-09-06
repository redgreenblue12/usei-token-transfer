use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, Bucket, ReadonlyBucket, bucket, bucket_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ACCOUNT_BALANCE_KEY: &[u8] = b"accountbalance";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountBalance {
    pub address: Addr,
    pub balance: u128,
}

pub fn resolver(storage: &mut dyn Storage) -> Bucket<AccountBalance> {
    bucket(storage, ACCOUNT_BALANCE_KEY)
}

pub fn resolver_read(storage: &dyn Storage) -> ReadonlyBucket<AccountBalance> {
    bucket_read(storage, ACCOUNT_BALANCE_KEY)
}