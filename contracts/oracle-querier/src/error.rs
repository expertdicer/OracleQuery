use cosmwasm_std::{ StdError};
use thiserror::Error;
use cosmos_sdk_proto::prost::{DecodeError, EncodeError};

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("OverflowError")]
    OverflowError {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("IllegalPrice")]
    IllegalPrice {},

    #[error("VerifyFail")]
    VerifyFail {},

    #[error("PriceNotExist")]
    PriceNotExist {},

    #[error("NotLatestData")]
    NotLatestData {},

    #[error("{0}")]
    Decode(#[from] DecodeError),
}
