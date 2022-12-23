use std::{ ops::Deref, vec};
use prost::bytes::Buf;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::traits::Message;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, coin, to_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, Storage, Uint128,
};

use crate::querier::UltraQuerier;
use crate::{error::ContractError, state::Rate};
use juno_querier::oracle_querier::{
    ExchangeRateResponse, ExecuteMsg, InstantiateMsg, OracleQuery, QueryBalanceRequest, QueryMsg,
    UltraQuery,
};
use std::io::Cursor;
use cosmos_sdk_proto::cosmos::bank::v1beta1::{
    QueryBalanceRequest as StargateQueryBalanceRequest,
    QueryBalanceResponse as StargateQueryBalanceResponse,
};
use cosmos_sdk_proto::cosmos::mint::v1beta1::{QueryInflationRequest, QueryInflationResponse};
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
        QueryMsg::BalanceStargate { address, denom } => {
            to_binary(&query_total_balance_stargate(deps, address, denom)?)
        }
        QueryMsg::InflationStargate {} => to_binary(&query_inflation_stargate(deps)?),
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

pub fn query_total_balance_stargate(deps: Deps, address: String, denom: String) -> StdResult<String> {
    let bin = StargateQueryBalanceRequest {
        address: address,
        denom: denom,
    }
    .encode_to_vec();

    let data = Binary::from(bin);

    let query: QueryRequest<Empty> = QueryRequest::Stargate {
        path: "/juno.bank.v1beta1.Query/Balance".to_string(),
        data,
    };

    let bin_res: Binary = deps.querier.query(&query)?;
    let res = StargateQueryBalanceResponse::decode(&*bin_res.to_vec())
        .map_err(ContractError::Decode)
        .unwrap();

    let coin = res.balance.unwrap();
    Ok(coin.amount)
}

pub fn query_inflation_stargate(deps: Deps) -> StdResult<Binary> {
    let bin = QueryInflationRequest {}.encode_to_vec();

    let data = Binary::from(bin);

    let query: QueryRequest<Empty> = QueryRequest::Stargate {
        path: "/juno.mint.Query/Inflation".to_string(),
        data,
    };
    let bin_res: Binary = deps.querier.query(&query)?;
    let res = QueryInflationResponse::decode(&*bin_res.to_vec())
        .map_err(ContractError::Decode)
        .unwrap();
    let inflation = to_binary(&res.inflation)?;
    
    Ok(inflation)
}
