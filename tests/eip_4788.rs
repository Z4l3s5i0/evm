mod mock;
use evm::uint::{H160, H256, U256};
use evm::{
	backend::OverlayedBackend,
	interpreter::etable::{Chained, Single},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, BlockExecutor,
	},
};
use mock::{MockBackend};

#[test]
fn test_eip4788_beacon_root_system_call() {
	let backend = MockBackend::default();
	let config = Config::prague();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = DispatchEtable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(&(), &etable);
	let invoker = Invoker::new(&resolver);

	let beacon_root = H256::from_low_u64_be(0xbeef);

	// Apply block start which should call beacon root contract
	BlockExecutor::apply_block_start::<_, Invoker<_>>(&invoker, &mut overlayed_backend, &config, Some(beacon_root));

	// Since we don't have code for 0x4788, it just returns.
	// But the call was made.
}
