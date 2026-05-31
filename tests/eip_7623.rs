mod mock;
use evm::uint::{H160, U256};
use evm::{
	backend::OverlayedBackend,
	interpreter::etable::{Chained, Single},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, TransactArgs, TransactArgsCallCreate,
		TransactValue,
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
fn test_eip7623_floor_gas_high_calldata() {
	let mut backend = MockBackend::default();
	let caller = H160::from_low_u64_be(1);
	backend.state.insert(
		caller,
		MockAccount {
			balance: U256::from(1_000_000_000),
			..Default::default()
		},
	);

	let config = Config::prague();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	// Large calldata: 10,000 bytes of zeros
	let data = vec![0u8; 10000];

	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: H160::from_low_u64_be(0x1234),
			data,
		},
		caller,
		value: U256::zero(),
		gas_limit: U256::from(200_000),
		gas_price: U256::from(1).into(),
		access_list: vec![],
		authorization_list: vec![],
		config: &config,
	};

	let result = transact(args, &mut overlayed_backend).expect("Transaction failed");

	// EIP-7623 floor gas for 10000 bytes of zero calldata:
	// tokens = 10000 * 4 = 40000
	// floor = 21000 + (tokens * 10 / 4) = 21000 + (40000 * 2.5) = 21000 + 100000 = 121000
	// Traditional intrinsic gas: 21000 + 10000 * 4 = 61000
	// Floor 121000 > Traditional 61000, so used gas should be at least 121000
	assert!(result.used_gas >= U256::from(121000));
}

#[test]
fn test_eip7623_floor_gas_low_calldata() {
	let mut backend = MockBackend::default();
	let caller = H160::from_low_u64_be(1);
	backend.state.insert(
		caller,
		MockAccount {
			balance: U256::from(1_000_000_000),
			..Default::default()
		},
	);

	let config = Config::prague();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	// Small calldata
	let data = vec![0u8; 100];

	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: H160::from_low_u64_be(0x1234),
			data,
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

	// EIP-7623 floor gas for 100 bytes of zero calldata:
	// tokens = 100 * 4 = 400
	// floor = 21000 + (400 * 10 / 4) = 21000 + 1000 = 22000
	// Traditional intrinsic gas: 21000 + 100 * 4 = 21400
	// Floor 22000 > Traditional 21400, used gas should be at least 22000
	assert!(result.used_gas >= U256::from(22000));
}
