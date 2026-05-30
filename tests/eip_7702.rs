mod mock;
use evm::uint::{H160, U256, U256Ext};
use evm::{
	backend::OverlayedBackend,
	interpreter::{
		etable::{Chained, Single},
	},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, TransactArgs, TransactArgsCallCreate,
		TransactValue, TransactValueCallCreate,
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
fn test_eip7702_delegation_gas() {
	let mut backend = MockBackend::default();
	
	// Target contract: simply STOP
	let target_address = H160::from_low_u64_be(0x1337);
	backend.state.insert(
		target_address,
		MockAccount {
			balance: U256::ZERO,
			code: vec![0x00], // STOP
			nonce: U256::ZERO,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);

	// Delegated account: 0xef0100 || target_address
	let delegated_address = H160::from_low_u64_be(0x7702);
	let mut delegation_code = vec![0xef, 0x01, 0x00];
	delegation_code.extend_from_slice(target_address.as_bytes());
	
	backend.state.insert(
		delegated_address,
		MockAccount {
			balance: U256::ZERO,
			code: delegation_code,
			nonce: U256::ZERO,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);

	// Caller
	let caller = H160::from_low_u64_be(1);
	backend.state.insert(
		caller,
		MockAccount {
			balance: U256::from(1_000_000_000),
			code: vec![],
			nonce: U256::ONE,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);

	let config = Config::prague();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: delegated_address,
			data: vec![],
		},
		caller,
		value: U256::ZERO,
		gas_limit: U256::from(100_000),
		gas_price: U256::from(1).into(),
		access_list: vec![],
		config: &config,
	};

	let result = transact(args, &mut overlayed_backend).expect("Transaction failed");
	
	// Intrinsic gas: 21000
	// 7702 delegation (cold): 2600
	// Total expected: 21000 + 2600 = 23600
	assert_eq!(result.used_gas, U256::from(23600));
}
