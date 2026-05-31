use alloc::vec::Vec;
use evm::uint::U256;
use evm::{
	GasMutState,
	interpreter::{ExitError, ExitResult},
};
use crate::PurePrecompile;

pub struct Bls12381G1Add;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G1Add {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let _ = gasometer.record_gas(U256::from(600u64));
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381G1Mul;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G1Mul {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let _ = gasometer.record_gas(U256::from(12000u64));
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381G1MultiExp;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G1MultiExp {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381G2Add;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G2Add {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let _ = gasometer.record_gas(U256::from(4500u64));
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381G2Mul;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G2Mul {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let _ = gasometer.record_gas(U256::from(55000u64));
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381G2MultiExp;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G2MultiExp {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381Pairing;
impl<G: GasMutState> PurePrecompile<G> for Bls12381Pairing {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381MapG1;
impl<G: GasMutState> PurePrecompile<G> for Bls12381MapG1 {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let _ = gasometer.record_gas(U256::from(110000u64));
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}

pub struct Bls12381MapG2;
impl<G: GasMutState> PurePrecompile<G> for Bls12381MapG2 {
	fn execute(&self, _input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let _ = gasometer.record_gas(U256::from(190000u64));
		(Err(ExitError::Other("Not implemented".into())), Vec::new())
	}
}
