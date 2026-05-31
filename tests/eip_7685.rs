mod mock;
use evm::uint::{H160, U256};
use evm::{
	backend::OverlayedBackend,
	interpreter::etable::{Chained, Single},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, BlockExecutor,
	},
};
use mock::{MockBackend};

#[test]
fn test_eip7685_request_collection() {
	let backend = MockBackend::default();
	let config = Config::prague();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = DispatchEtable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(&(), &etable);
	let invoker = Invoker::new(&resolver);

	// Mock system contracts with code that calls PUSH_REQUEST (we need an opcode or precompile for this)
	// Actually, EIP-7685 system contracts usually trigger the host to push requests.
	// In our implementation, BlockExecutor calls these contracts.
	// Let's mock the WITHDRAWAL_REQUEST_ADDRESS with code that uses a hypothetical opcode if it existed,
	// or just manually push to substate if we can.

	// Since we don't have a PUSH_REQUEST opcode, the system contracts must be precompiles or
	// we use the fact that they are called.
	// But our `apply_block_end` just calls them.

	// Let's verify that `apply_block_end` returns requests collected in the backend.
	use evm::interpreter::runtime::RuntimeBackend;
	overlayed_backend.push_request(1, vec![1, 2, 3]).unwrap();
	overlayed_backend.push_request(2, vec![4, 5, 6]).unwrap();

	let requests = BlockExecutor::apply_block_end::<_, Invoker<_>>(&invoker, &mut overlayed_backend, &config);

	assert_eq!(requests.len(), 2);
	assert_eq!(requests[0], (1, vec![1, 2, 3]));
	assert_eq!(requests[1], (2, vec![4, 5, 6]));
}
