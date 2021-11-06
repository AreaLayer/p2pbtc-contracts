use crate::errors::OfferError;
use cosmwasm_std::{Addr, StdError, StdResult};

pub fn assert_ownership(caller: Addr, owner: Addr) -> Result<(), OfferError> {
    if caller.eq(&owner) {
        Ok(())
    } else {
        Err(OfferError::Unauthorized { owner, caller })
    }
}

pub fn assert_min_g_max(min: u64, max: u64) -> Result<(), OfferError> {
    if min >= max {
        Err(OfferError::Std(StdError::generic_err(
            "Min amount must be greater than Max amount.",
        )))
    } else {
        Ok(())
    }
}
