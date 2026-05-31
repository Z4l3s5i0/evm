use evm_interpreter::uint::{H160, U256, H256};
use evm_interpreter::runtime::{RuntimeBackend, RuntimeEnvironment};
use crate::standard::Config;
use alloc::vec::Vec;

/// Standard block executor.
pub struct BlockExecutor;

impl BlockExecutor {
    /// EIP-2935: History storage contract address.
    pub const HISTORY_STORAGE_ADDRESS: H160 = H160([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x29, 0x35,
    ]);

    /// Apply block start transitions (EIP-2935).
    pub fn apply_block_start<H: RuntimeBackend + RuntimeEnvironment>(
        handler: &mut H,
        config: &Config,
    ) {
        if config.eip2935_historical_block_hashes {
            let parent_hash = handler.block_hash(handler.block_number().saturating_sub(U256::from(1)));
            let index = handler.block_number().saturating_sub(U256::from(1)) % U256::from(8192);
            let mut index_bytes = [0u8; 32];
            index.to_big_endian(&mut index_bytes);

            let _ = handler.set_storage(
                Self::HISTORY_STORAGE_ADDRESS,
                H256::from_slice(&index_bytes),
                parent_hash
            );
        }
    }

    /// Apply block end transitions (EIP-7685).
    pub fn apply_block_end<H: RuntimeBackend + RuntimeEnvironment>(
        handler: &mut H,
    ) -> Vec<(u8, Vec<u8>)> {
        handler.requests()
    }
}
