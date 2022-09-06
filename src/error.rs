use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Withdrawal quantity {withdraw_quantity:?} exceeds balance {balance:?}")]
    InsufficientFunds {
        withdraw_quantity: u128,
        balance: u128,
    },

    #[error("Can't withdraw no coins {withdraw_quantity:?}")]
    EmptyWithdrawQuantity { withdraw_quantity: u128 },

    #[error("Can't cover the contract fee in the sent amount {send_quantity:?}")]
    CannotCoverFee { send_quantity: u128 },

    // TODO: Could use better string formatting here.
    #[error("The percent fee must be below 100% but is {percent_fee:?}")]
    PercentFeeTooLarge { percent_fee: u128 },
}
