use std::{ ops::Deref, vec};
use prost::bytes::Buf;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
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
use cosmos_sdk_proto::ibc::core::channel::v1::{QueryNextSequenceSendRequest, QueryNextSequenceSendResponse};
// use ibc_proto::ibc::core::channel::v1::{QueryNextSequenceSendResponse, QueryNextSequenceSendRequest};
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
        QueryMsg::NextSequenceSend { port_id, channel_id } =>  to_binary(&query_next_sequence_send(deps, port_id, channel_id)?),
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



pub fn query_next_sequence_send(deps: Deps, port_id: String, channel_id: String) -> StdResult<Binary> {
    let bin = QueryNextSequenceSendRequest {
        port_id,
        channel_id,
    };

    let req: Binary = to_binary(&bin)?;
    let query: QueryRequest<Empty> = QueryRequest::Stargate {
        path: "/ibc.core.channel.v1.Query/NextSequenceSend".to_string(),
        data: req,
    };
    let bin_res: Binary = deps.querier.query(&query)?;
    let res = to_binary(&bin_res)?;
    
    Ok(res)
}