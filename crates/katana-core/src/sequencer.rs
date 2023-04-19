use std::{collections::HashMap, sync::Mutex};

use blockifier::{block_context::BlockContext, state::cached_state::CachedState};
use starknet_api::{
    block::{BlockNumber, BlockTimestamp},
    core::{ChainId, ContractAddress, PatriciaKey},
    hash::StarkHash,
    patricia_key,
};

use crate::state::DictStateReader;

pub const DEFAULT_GAS_PRICE: u128 = 100 * u128::pow(10, 9); // Given in units of wei.
pub const SEQUENCER_ADDRESS: &str = "0x69";
pub const FEE_ERC20_CONTRACT_ADDRESS: &str = "0x420";

pub struct KatanaSequencer {
    pub block_context: BlockContext,
    pub state: Mutex<CachedState<DictStateReader>>,
}

impl KatanaSequencer {
    pub fn new() -> Self {
        Self {
            block_context: BlockContext {
                chain_id: ChainId("KATANA".to_string()),
                block_number: BlockNumber::default(),
                block_timestamp: BlockTimestamp::default(),
                sequencer_address: ContractAddress(patricia_key!(SEQUENCER_ADDRESS)),
                fee_token_address: ContractAddress(patricia_key!(FEE_ERC20_CONTRACT_ADDRESS)),
                cairo_resource_fee_weights: HashMap::from([
                    (String::from("n_steps"), 1_f64),
                    (String::from("pedersen_builtin"), 1_f64),
                    (String::from("range_check_builtin"), 1_f64),
                    (String::from("ecdsa_builtin"), 1_f64),
                    (String::from("bitwise_builtin"), 1_f64),
                    (String::from("poseidon_builtin"), 1_f64),
                    (String::from("output_builtin"), 1_f64),
                    (String::from("ec_op_builtin"), 1_f64),
                ]),
                gas_price: DEFAULT_GAS_PRICE,
                invoke_tx_max_n_steps: 1_000_000,
                validate_max_n_steps: 1_000_000,
            },
            state: Mutex::new(CachedState::new(DictStateReader::new())),
        }
    }
}

impl Default for KatanaSequencer {
    fn default() -> Self {
        Self::new()
    }
}
