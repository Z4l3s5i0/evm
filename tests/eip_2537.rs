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

use evm_precompile::StandardPrecompileSet;

fn transact_with_precompiles(
	args: TransactArgs,
	overlayed_backend: &mut OverlayedBackend<MockBackend>,
) -> Result<TransactValue, evm::interpreter::ExitError> {
	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = DispatchEtable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(&StandardPrecompileSet, &etable);
	let invoker = Invoker::new(&resolver);

	evm::transact(args.clone(), Some(4), overlayed_backend, &invoker)
}

#[test]
fn test_eip2537_bls_g1_add() {
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

	// G1 addition precompile address: 0x0b
	let precompile_addr = H160::from_low_u64_be(0x0b);

	// G1 points are 128 bytes each.
	// Input must be exactly 256 bytes.
	let data = vec![0u8; 256];

	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: precompile_addr,
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

	let result = transact_with_precompiles(args, &mut overlayed_backend).expect("Transaction failed");
	match result.call_create {
		evm::standard::TransactValueCallCreate::Call { succeed, retval } => {
			assert_eq!(succeed, evm::interpreter::ExitSucceed::Returned);
			assert_eq!(retval, vec![0u8; 128]);
		}
		_ => panic!("Expected Call"),
	}
}

#[test]
fn test_eip2537_bls_pairing_empty() {
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

	// Pairing precompile address: 0x11
	let precompile_addr = H160::from_low_u64_be(0x11);

	// Pairing input must be multiple of 384 bytes.
	// Empty input is not allowed in our implementation.
	// 384 bytes of zeros (one pair of identity points).
	let data = vec![0u8; 384];

	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: precompile_addr,
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

	let result = transact_with_precompiles(args, &mut overlayed_backend).expect("Transaction failed");
	match result.call_create {
		evm::standard::TransactValueCallCreate::Call { succeed, retval } => {
			assert_eq!(succeed, evm::interpreter::ExitSucceed::Returned);
			let mut expected = vec![0u8; 32];
			expected[31] = 1;
			assert_eq!(retval, expected);
		}
		_ => panic!("Expected Call"),
	}
}
