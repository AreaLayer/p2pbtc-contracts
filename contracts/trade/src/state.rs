use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use localterra_protocol::trade::TradeData;

pub static STATE_KEY: &[u8] = b"state";

pub fn state(storage: &mut dyn Storage) -> Singleton<TradeData> {
    singleton(storage, STATE_KEY)
}

pub fn state_read(storage: &dyn Storage) -> ReadonlySingleton<TradeData> {
    singleton_read(storage, STATE_KEY)
}
