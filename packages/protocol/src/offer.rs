use super::constants::OFFERS_KEY;
use crate::currencies::FiatCurrency;
use crate::errors::OfferError;
use crate::trade::State as TradeState;
use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{self};

pub static CONFIG_KEY: &[u8] = b"config";
pub const OFFERS: Map<&[u8], Offer> = Map::new(OFFERS_KEY);

///Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OfferMsg {
    pub offer_type: OfferType,
    pub fiat_currency: FiatCurrency,
    pub min_amount: u64,
    pub max_amount: u64, // TODO change to Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Create {
        offer: OfferMsg,
    },
    Pause {
        id: u64,
    },
    Activate {
        id: u64,
    },
    Update {
        id: u64,
        offer: OfferMsg,
    },
    NewTrade {
        offer_id: u64,
        ust_amount: String,
        counterparty: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    Offers {
        fiat_currency: FiatCurrency,
    },
    OffersPage {
        fiat_currency: FiatCurrency,
        last_value: Vec<u8>,
        limit: usize,
    },
    Offer {
        id: u64,
    },
    Trades {
        maker: String,
    },
}

///Data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub factory_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub offers_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Offer {
    pub id: u64,
    pub owner: Addr,
    pub offer_type: OfferType,
    pub fiat_currency: FiatCurrency,
    pub min_amount: Uint128,
    pub max_amount: Uint128,
    pub state: OfferState,
}

pub struct OfferModel<'a> {
    pub offer: Offer,
    pub storage: &'a mut dyn Storage,
}

impl OfferModel<'_> {
    pub fn store(storage: &mut dyn Storage, offer: &Offer) -> StdResult<()> {
        OFFERS.save(storage, &offer.id.to_be_bytes(), &offer)
    }

    pub fn fetch(storage: &mut dyn Storage, id: &u64) -> Offer {
        OFFERS
            .may_load(storage, &id.to_be_bytes())
            .unwrap_or_default()
            .unwrap()
    }

    pub fn create(storage: &mut dyn Storage, offer: Offer) -> OfferModel {
        OfferModel::store(storage, &offer);
        OfferModel { offer, storage }
    }

    pub fn save<'a>(self) -> Offer {
        OfferModel::store(self.storage, &self.offer);
        self.offer
    }

    pub fn may_load<'a>(storage: &'a mut dyn Storage, id: &u64) -> OfferModel<'a> {
        let offer_model = OfferModel {
            offer: OfferModel::fetch(storage, &id),
            storage,
        };
        return offer_model;
    }

    pub fn activate(&mut self) -> Result<&Offer, OfferError> {
        match self.offer.state {
            OfferState::Paused => {
                self.offer.state = OfferState::Active;
                OfferModel::store(self.storage, &self.offer);
                Ok(&self.offer)
            }
            OfferState::Active => Err(OfferError::InvalidStateChange {
                from: self.offer.state.clone(),
                to: OfferState::Active,
            }),
        }
    }

    pub fn pause(&mut self) -> Result<&Offer, OfferError> {
        match self.offer.state {
            OfferState::Active => {
                self.offer.state = OfferState::Paused;
                OfferModel::store(self.storage, &self.offer);
                Ok(&self.offer)
            }
            OfferState::Paused => Err(OfferError::InvalidStateChange {
                from: self.offer.state.clone(),
                to: OfferState::Paused,
            }),
        }
    }

    pub fn update(&mut self, msg: OfferMsg) -> &Offer {
        self.offer.offer_type = msg.offer_type;
        self.offer.fiat_currency = msg.fiat_currency;
        self.offer.min_amount = Uint128::from(msg.min_amount);
        self.offer.max_amount = Uint128::from(msg.max_amount);
        OfferModel::store(self.storage, &self.offer);
        &self.offer
        // self.save()
        //     ^^^^ move occurs because `*self` has type `OfferModel<'_>`, which does not implement the `Copy` trait
    }

    pub fn query_all_offers(
        storage: &dyn Storage,
        fiat_currency: FiatCurrency,
    ) -> StdResult<Vec<Offer>> {
        let result: Vec<Offer> = OFFERS
            .range(storage, None, None, Order::Ascending)
            .flat_map(|item| item.and_then(|(_, offer)| Ok(offer)))
            .filter(|offer| offer.fiat_currency == fiat_currency)
            .collect();

        Ok(result)
    }

    pub fn fetch(
        storage: &dyn Storage,
        fiat_currency: FiatCurrency,
        last_value: Vec<u8>,
        limit: usize,
    ) -> StdResult<Vec<Offer>> {
        let result: Vec<Offer> = OFFERS
            .range(
                storage,
                Some(Bound::Exclusive(last_value)),
                None,
                Order::Ascending,
            )
            .take(limit)
            .flat_map(|item| item.and_then(|(_, offer)| Ok(offer)))
            .filter(|offer| offer.fiat_currency == fiat_currency)
            .collect();

        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeInfo {
    pub trade: TradeState,
    pub offer: Offer,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OfferType {
    Buy,
    Sell,
}
impl fmt::Display for OfferType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OfferState {
    Active,
    Paused,
}
