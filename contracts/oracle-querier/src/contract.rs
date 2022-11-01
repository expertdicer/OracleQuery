use std::{ops::Deref, vec};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, Storage, Uint128,
};

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::querier::UltraQuerier;
use juno_stable::oracle_querier::{
    ExchangeRateResponse, ExecuteMsg, InstantiateMsg, OracleQuery, QueryMsg, UltraQuery,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:active-pool";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const NATIVE_JUNO_DENOM: &str = "ujuno";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ExchangeRate { denom } => to_binary(&query_exchange_rate(deps, denom)?),
    }
}

pub fn query_exchange_rate(deps: Deps, denom: String) -> StdResult<Decimal> {
    let query: UltraQuery = UltraQuery::Oracle(OracleQuery::ExchangeRate {
        denom: denom.into(),
    });
    let request: QueryRequest<UltraQuery> = UltraQuery::into(query);
    let querier: QuerierWrapper<UltraQuery> =
        QuerierWrapper::<UltraQuery>::new(deps.querier.deref());
    let exchangerate: ExchangeRateResponse = querier.query(&request)?;
    Ok(exchangerate.rate)
}
