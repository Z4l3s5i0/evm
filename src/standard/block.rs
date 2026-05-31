use evm_interpreter::uint::{H160, U256, H256, U256Ext};
use evm_interpreter::runtime::{RuntimeBackend, RuntimeEnvironment};
use evm_interpreter::Interpreter;
use crate::standard::Config;
use crate::invoker::{Invoker, InvokerControl, InvokerExit};
use crate::standard::invoker::{TransactArgs, TransactArgsCallCreate, TransactGasPrice};
use alloc::vec::Vec;

/// Standard block executor.
pub struct BlockExecutor;

impl BlockExecutor {
    /// EIP-2935: History storage contract address.
    pub const HISTORY_STORAGE_ADDRESS: H160 = H160([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x29, 0x35,
    ]);

    /// EIP-4788: Beacon root contract address.
    pub const BEACON_ROOT_ADDRESS: H160 = H160([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x47, 0x88,
    ]);

    /// EIP-7002: Withdrawal requests contract address.
    pub const WITHDRAWAL_REQUEST_ADDRESS: H160 = H160([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x70, 0x02,
    ]);

    /// EIP-7251: Consolidation requests contract address.
    pub const CONSOLIDATION_REQUEST_ADDRESS: H160 = H160([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x72, 0x51,
    ]);

    /// EIP-6110: Deposit requests contract address.
    pub const DEPOSIT_REQUEST_ADDRESS: H160 = H160([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x61, 0x10,
    ]);

    /// Apply block start transitions (EIP-2935, EIP-4788).
    pub fn apply_block_start<'config, H, I>(
        invoker: &I,
        handler: &mut H,
        config: &'config Config,
        beacon_root: Option<H256>,
    ) where
        H: RuntimeBackend + RuntimeEnvironment,
        I: Invoker<H, TransactArgs = TransactArgs<'config>>,
        <I::Interpreter as Interpreter<H>>::State: Clone,
    {
        // EIP-2935: History storage contract
        if config.eip2935_historical_block_hashes {
            let parent_hash = handler.block_hash(handler.block_number().saturating_sub(U256::from(1)));
            let index = handler.block_number().saturating_sub(U256::from(parent_hash.is_zero() as u64)) % U256::from(8192);
            let mut index_bytes = [0u8; 32];
            index.to_big_endian(&mut index_bytes);

            let _ = handler.set_storage(
                Self::HISTORY_STORAGE_ADDRESS,
                H256::from_slice(&index_bytes),
                parent_hash
            );
        }

        // EIP-4788: Beacon root contract
        if config.eip4788_beacon_root {
            if let Some(root) = beacon_root {
                let args = TransactArgs {
                    caller: H160::default(),
                    call_create: TransactArgsCallCreate::Call {
                        address: Self::BEACON_ROOT_ADDRESS,
                        data: root.as_bytes().to_vec(),
                    },
                    value: U256::ZERO,
                    gas_limit: U256::from(30_000_000), // Max gas for system call
                    gas_price: TransactGasPrice::Legacy(U256::ZERO),
                    access_list: Vec::new(),
                    authorization_list: Vec::new(),
                    config,
                };

                if let Ok((invoke, control)) = invoker.new_transact(args, handler) {
                    if let InvokerControl::Enter(mut machine) = control {
                        let exit = machine.run(handler);
                        if let evm_interpreter::Capture::Exit(res) = exit {
                            let machine_state = machine.state();
                            let _ = invoker.finalize_transact(&invoke, InvokerExit {
                                result: res,
                                substate: Some(machine_state.clone()),
                                retval: Vec::new(),
                                instruction_count: 0,
                            }, handler);
                        }
                    }
                }
            }
        }
    }

    /// Apply block end transitions (EIP-7685, EIP-7002, EIP-7251, EIP-6110).
    pub fn apply_block_end<'config, H, I>(
        invoker: &I,
        handler: &mut H,
        config: &'config Config,
    ) -> Vec<(u8, Vec<u8>)>
    where
        H: RuntimeBackend + RuntimeEnvironment,
        I: Invoker<H, TransactArgs = TransactArgs<'config>>,
        <I::Interpreter as Interpreter<H>>::State: Clone,
    {
        if config.eip7685_execution_layer_requests {
            let addresses = [
                Self::WITHDRAWAL_REQUEST_ADDRESS,
                Self::CONSOLIDATION_REQUEST_ADDRESS,
                Self::DEPOSIT_REQUEST_ADDRESS,
            ];

            for addr in addresses {
                let args = TransactArgs {
                    caller: H160::default(),
                    call_create: TransactArgsCallCreate::Call {
                        address: addr,
                        data: Vec::new(),
                    },
                    value: U256::ZERO,
                    gas_limit: U256::from(30_000_000),
                    gas_price: TransactGasPrice::Legacy(U256::ZERO),
                    access_list: Vec::new(),
                    authorization_list: Vec::new(),
                    config,
                };

                if let Ok((invoke, control)) = invoker.new_transact(args, handler) {
                    if let InvokerControl::Enter(mut machine) = control {
                        let res = machine.run(handler);
                        if let evm_interpreter::Capture::Exit(res) = res {
                             let machine_state = machine.state();
                             let _ = invoker.finalize_transact(&invoke, InvokerExit {
                                result: res,
                                substate: Some(machine_state.clone()),
                                retval: Vec::new(),
                                instruction_count: 0,
                            }, handler);
                        }
                    }
                }
            }
        }

        handler.requests()
    }
}
