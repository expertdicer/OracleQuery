use std::{ops::Deref, vec};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, coin, to_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, Storage, Uint128,
};

use cw2::set_contract_version;

use crate::querier::UltraQuerier;
use crate::{error::ContractError, state::Rate};
use juno_stable::oracle_querier::{
    ExchangeRateResponse, ExecuteMsg, InstantiateMsg, OracleQuery, QueryBalanceRequest, QueryMsg,
    UltraQuery,
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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetExchangeRate { denom } => get_exchange_rate(deps, denom),
    }
}

pub fn get_exchange_rate(deps: DepsMut, denom: String) -> Result<Response, ContractError> {
    let query: UltraQuery = UltraQuery::Oracle(OracleQuery::ExchangeRate {
        denom: denom.clone(),
    });
    let request: QueryRequest<UltraQuery> = UltraQuery::into(query);
    let querier: QuerierWrapper<UltraQuery> =
        QuerierWrapper::<UltraQuery>::new(deps.querier.deref());
    let exchangerate: ExchangeRateResponse = querier.query(&request)?;
    Rate.save(deps.storage, denom.clone(), &exchangerate.rate)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "get_rate"),
        attr("denom", denom),
        attr("rate", exchangerate.rate.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ExchangeRate { denom } => to_binary(&query_exchange_rate(deps, denom)?),
        QueryMsg::ExchangeRateStarGate { address, denom } => {
            to_binary(&query_exchange_rate_stargate(deps, address, denom)?)
        }
    }
}

pub fn query_exchange_rate(deps: Deps, denom: String) -> StdResult<Decimal> {
    let rate = Rate.load(deps.storage, denom)?;
    Ok(rate)
}

pub fn query_exchange_rate_stargate(
    deps: Deps,
    address: String,
    denom: String,
) -> StdResult<Decimal> {
    deps.api.addr_validate(&address)?;
    let query_request = QueryBalanceRequest {
        address: address,
        denom: denom,
    };
    let data: Binary = to_binary(&query_request)?;
    let request = QueryRequest::Stargate {
        path: "/juno.oracle.v1.Query/ExchangeRates".to_string(),
        data: data,
    };
    let exchange_rate_res: ExchangeRateResponse = deps.querier.query(&request)?;
    Ok(exchange_rate_res.rate)
}
