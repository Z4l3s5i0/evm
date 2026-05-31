mod mock;
use evm::uint::{H160, H256, U256, U256Ext};
use evm::{
	backend::OverlayedBackend,
	interpreter::etable::{Chained, Single},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, TransactArgs, TransactArgsCallCreate,
		TransactValue, BlockExecutor,
	},
};
use mock::{MockAccount, MockBackend};

fn transact(
	args: TransactArgs,
	overlayed_backend: &mut OverlayedBackend<MockBackend>,
) -> Result<TransactValue, evm::interpreter::ExitError> {
	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = DispatchEtable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(&(), &etable);
	let invoker = Invoker::new(&resolver);

	evm::transact(args.clone(), Some(4), overlayed_backend, &invoker)
}

#[test]
fn test_eip2935_blockhash_history_contract() {
	let mut backend = MockBackend::default();
	let caller = H160::from_low_u64_be(1);
	backend.state.insert(
		caller,
		MockAccount {
			balance: U256::from(1_000_000_000),
			..Default::default()
		},
	);

	// Block number 1000
	backend.block_number = U256::from(1000);

	// Set history storage at 0x2935
	let history_addr = BlockExecutor::HISTORY_STORAGE_ADDRESS;
	let block_to_query = U256::from(500);
	let block_hash = H256::from_low_u64_be(0xdeadbeef);

	// Key is block number % 8192
	let index = (block_to_query % U256::from(8192)).as_u64();
	let storage_key = H256::from_low_u64_be(index);

	backend.state.entry(history_addr).or_default().storage.insert(storage_key, block_hash);

	let config = Config::prague();
	// Move overlayed_backend creation later

	// Code to call BLOCKHASH for block 500
	// PUSH2 500, BLOCKHASH, PUSH1 0, MSTORE, RETURN(0, 32)
	let code = vec![
		0x61, 0x01, 0xf4, // PUSH2 500
		0x40,             // BLOCKHASH
		0x60, 0x00,       // PUSH1 0
		0x52,             // MSTORE
		0x60, 0x20,       // PUSH1 32
		0x60, 0x00,       // PUSH1 0
		0xf3,             // RETURN
	];

	let contract_addr = H160::from_low_u64_be(0x1234);
	// Since MockBackend has state public.
	backend.state.insert(contract_addr, MockAccount {
		code,
		..Default::default()
	});
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: contract_addr,
			data: vec![],
		},
		caller,
		value: U256::zero(),
		gas_limit: U256::from(100_000),
		gas_price: U256::from(1).into(),
		access_list: vec![],
		authorization_list: vec![],
		config: &config,
	};

	let result = transact(args, &mut overlayed_backend).expect("Transaction failed");
	match result.call_create {
		evm::standard::TransactValueCallCreate::Call { succeed, retval } => {
			assert_eq!(succeed, evm::interpreter::ExitSucceed::Returned);
			assert_eq!(H256::from_slice(&retval), block_hash);
		}
		_ => panic!("Expected Call"),
	}
}

#[test]
fn test_eip2935_system_call() {
	let mut backend = MockBackend::default();
	backend.block_number = U256::from(10);
	let parent_hash = H256::from_low_u64_be(0xabc);
	backend.block_hashes.insert(U256::from(9), parent_hash);

	let config = Config::prague();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = DispatchEtable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(&(), &etable);
	let invoker = Invoker::new(&resolver);

	BlockExecutor::apply_block_start::<_, Invoker<_>>(&invoker, &mut overlayed_backend, &config, None);

	// Verify that history contract at 0x2935 received the parent hash in its data
	// Since we didn't provide code, it just returns.
	// But we can check that a call was made if we had a more sophisticated mock.
	// For now, let's just make sure it doesn't crash.
}
