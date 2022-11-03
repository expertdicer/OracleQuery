use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage};
use cw_storage_plus::{Item, Map};

pub const Rate: Map<String, Decimal> = Map::new("rate");
