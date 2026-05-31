use alloc::vec;
use alloc::vec::Vec;
use ark_bls12_381::{Fq, Fq2, G1Affine, G2Affine, G1Projective, G2Projective, Bls12_381};
use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
use ark_ec::hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurve};
use ark_ec::pairing::Pairing;
use ark_ff::{BigInteger, PrimeField, Zero, One};
use evm::uint::U256;
use evm::{
	GasMutState,
	interpreter::{ExitError, ExitException, ExitResult, ExitSucceed},
};
use crate::PurePrecompile;

fn read_fp(input: &[u8]) -> Result<Fq, ExitError> {
	if input.len() < 64 {
		return Err(ExitException::Other("invalid fp length".into()).into());
	}
	for i in 0..16 {
		if input[i] != 0 {
			return Err(ExitException::Other("invalid fp encoding".into()).into());
		}
	}
	let f = Fq::from_be_bytes_mod_order(&input[16..64]);
	Ok(f)
}

fn read_fp2(input: &[u8]) -> Result<Fq2, ExitError> {
	if input.len() < 128 {
		return Err(ExitException::Other("invalid fp2 length".into()).into());
	}
	let c0 = read_fp(&input[0..64])?;
	let c1 = read_fp(&input[64..128])?;
	Ok(Fq2::new(c0, c1))
}

fn read_g1_point(input: &[u8]) -> Result<G1Affine, ExitError> {
	if input.len() < 128 {
		return Err(ExitException::Other("invalid g1 length".into()).into());
	}
	let x = read_fp(&input[0..64])?;
	let y = read_fp(&input[64..128])?;

	if x.is_zero() && y.is_zero() {
		Ok(G1Affine::identity())
	} else {
		let point = G1Affine::new(x, y);
		if !point.is_on_curve() {
			return Err(ExitException::Other("point not on curve".into()).into());
		}
		Ok(point)
	}
}

fn read_g2_point(input: &[u8]) -> Result<G2Affine, ExitError> {
	if input.len() < 256 {
		return Err(ExitException::Other("invalid g2 length".into()).into());
	}
	let x = read_fp2(&input[0..128])?;
	let y = read_fp2(&input[128..256])?;

	if x.is_zero() && y.is_zero() {
		Ok(G2Affine::identity())
	} else {
		let point = G2Affine::new(x, y);
		if !point.is_on_curve() {
			return Err(ExitException::Other("point not on curve".into()).into());
		}
		Ok(point)
	}
}

fn encode_g1_point(point: G1Affine) -> Vec<u8> {
	let mut out = vec![0u8; 128];
	if !point.is_zero() {
		let (x, y) = point.xy().unwrap();
		let x_bytes = x.into_bigint().to_bytes_be();
		let y_bytes = y.into_bigint().to_bytes_be();
		out[64 - x_bytes.len()..64].copy_from_slice(&x_bytes);
		out[128 - y_bytes.len()..128].copy_from_slice(&y_bytes);
	}
	out
}

fn encode_g2_point(point: G2Affine) -> Vec<u8> {
	let mut out = vec![0u8; 256];
	if !point.is_zero() {
		let (x, y) = point.xy().unwrap();
		let x_c0_bytes = x.c0.into_bigint().to_bytes_be();
		let x_c1_bytes = x.c1.into_bigint().to_bytes_be();
		let y_c0_bytes = y.c0.into_bigint().to_bytes_be();
		let y_c1_bytes = y.c1.into_bigint().to_bytes_be();

		out[64 - x_c0_bytes.len()..64].copy_from_slice(&x_c0_bytes);
		out[128 - x_c1_bytes.len()..128].copy_from_slice(&x_c1_bytes);
		out[192 - y_c0_bytes.len()..192].copy_from_slice(&y_c0_bytes);
		out[256 - y_c1_bytes.len()..256].copy_from_slice(&y_c1_bytes);
	}
	out
}

fn read_scalar(input: &[u8]) -> ark_bls12_381::Fr {
	let mut bytes = [0u8; 32];
	let len = core::cmp::min(input.len(), 32);
	bytes[32 - len..32].copy_from_slice(&input[0..len]);
	ark_bls12_381::Fr::from_be_bytes_mod_order(&bytes)
}

pub struct Bls12381G1Add;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G1Add {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		if let Err(e) = gasometer.record_gas(U256::from(600u64)) {
			return (Err(e), Vec::new());
		}
		if input.len() != 256 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let p1 = match read_g1_point(&input[0..128]) {
			Ok(p) => p,
			Err(e) => return (Err(e), Vec::new()),
		};
		let p2 = match read_g1_point(&input[128..256]) {
			Ok(p) => p,
			Err(e) => return (Err(e), Vec::new()),
		};
		let res = (p1 + p2).into_affine();
		(ExitSucceed::Returned.into(), encode_g1_point(res))
	}
}

pub struct Bls12381G1Mul;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G1Mul {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		if let Err(e) = gasometer.record_gas(U256::from(12000u64)) {
			return (Err(e), Vec::new());
		}
		if input.len() != 160 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let p = match read_g1_point(&input[0..128]) {
			Ok(p) => p,
			Err(e) => return (Err(e), Vec::new()),
		};
		if !p.is_in_correct_subgroup_assuming_on_curve() {
			return (Err(ExitException::Other("point not in correct subgroup".into()).into()), Vec::new());
		}
		let s = read_scalar(&input[128..160]);
		let res = (p * s).into_affine();
		(ExitSucceed::Returned.into(), encode_g1_point(res))
	}
}

const G1_MSM_DISCOUNT: [u64; 128] = [
	1000, 949, 848, 797, 764, 750, 738, 728, 719, 712, 705, 698, 692, 687, 682, 677, 673, 669, 665,
	661, 658, 654, 651, 648, 645, 642, 640, 637, 635, 632, 630, 627, 625, 623, 621, 619, 617, 615,
	613, 611, 609, 608, 606, 604, 603, 601, 599, 598, 596, 595, 593, 592, 591, 589, 588, 586, 585,
	584, 582, 581, 580, 579, 577, 576, 575, 574, 573, 572, 570, 569, 568, 567, 566, 565, 564, 563,
	562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 551, 550, 549, 548, 547, 547, 546, 545,
	544, 543, 542, 541, 540, 540, 539, 538, 537, 536, 536, 535, 534, 533, 532, 532, 531, 530, 529,
	528, 528, 527, 526, 525, 525, 524, 523, 522, 522, 521, 520, 520, 519,
];

fn get_msm_gas(k: usize, mul_cost: u64, discounts: &[u64], max_discount: u64) -> u64 {
	if k == 0 { return 0; }
	let discount = if k <= discounts.len() { discounts[k - 1] } else { max_discount };
	(k as u64 * mul_cost * discount) / 1000
}

pub struct Bls12381G1MultiExp;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G1MultiExp {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let k = input.len() / 160;
		if k == 0 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let gas_cost = 320000 + get_msm_gas(k, 12000, &G1_MSM_DISCOUNT, 519);
		if let Err(e) = gasometer.record_gas(U256::from(gas_cost)) {
			return (Err(e), Vec::new());
		}
		let mut points = Vec::with_capacity(k);
		let mut scalars = Vec::with_capacity(k);
		for i in 0..k {
			let point_data = &input[i * 160..i * 160 + 128];
			let scalar_data = &input[i * 160 + 128..i * 160 + 160];
			let p = match read_g1_point(point_data) {
				Ok(p) => p,
				Err(e) => return (Err(e), Vec::new()),
			};
			if !p.is_in_correct_subgroup_assuming_on_curve() {
				return (Err(ExitException::Other("point not in correct subgroup".into()).into()), Vec::new());
			}
			let s = read_scalar(scalar_data);
			points.push(p);
			scalars.push(s);
		}
		let res = G1Projective::msm_unchecked(&points, &scalars).into_affine();
		(ExitSucceed::Returned.into(), encode_g1_point(res))
	}
}

pub struct Bls12381G2Add;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G2Add {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		if let Err(e) = gasometer.record_gas(U256::from(4500u64)) {
			return (Err(e), Vec::new());
		}
		if input.len() != 512 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let p1 = match read_g2_point(&input[0..256]) {
			Ok(p) => p,
			Err(e) => return (Err(e), Vec::new()),
		};
		let p2 = match read_g2_point(&input[256..512]) {
			Ok(p) => p,
			Err(e) => return (Err(e), Vec::new()),
		};
		let res = (p1 + p2).into_affine();
		(ExitSucceed::Returned.into(), encode_g2_point(res))
	}
}

pub struct Bls12381G2Mul;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G2Mul {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		if let Err(e) = gasometer.record_gas(U256::from(30000u64)) {
			return (Err(e), Vec::new());
		}
		if input.len() != 288 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let p = match read_g2_point(&input[0..256]) {
			Ok(p) => p,
			Err(e) => return (Err(e), Vec::new()),
		};
		if !p.is_in_correct_subgroup_assuming_on_curve() {
			return (Err(ExitException::Other("point not in correct subgroup".into()).into()), Vec::new());
		}
		let s = read_scalar(&input[256..288]);
		let res = (p * s).into_affine();
		(ExitSucceed::Returned.into(), encode_g2_point(res))
	}
}

const G2_MSM_DISCOUNT: [u64; 128] = [
	1000, 1000, 923, 884, 855, 832, 812, 796, 782, 770, 759, 749, 740, 732, 724, 717, 711, 704, 699,
	693, 688, 683, 679, 674, 670, 666, 663, 659, 655, 652, 649, 646, 643, 640, 637, 634, 632, 629,
	627, 624, 622, 620, 618, 615, 613, 611, 609, 607, 606, 604, 602, 600, 598, 597, 595, 593, 592,
	590, 589, 587, 586, 584, 583, 582, 580, 579, 578, 576, 575, 574, 573, 571, 570, 569, 568, 567,
	566, 565, 563, 562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 552, 551, 550, 549, 548,
	547, 546, 545, 545, 544, 543, 542, 541, 541, 540, 539, 538, 537, 537, 536, 535, 535, 534, 533,
	532, 532, 531, 530, 530, 529, 528, 528, 527, 526, 526, 525, 524, 524,
];

pub struct Bls12381G2MultiExp;
impl<G: GasMutState> PurePrecompile<G> for Bls12381G2MultiExp {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let k = input.len() / 288;
		if k == 0 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let gas_cost = 750000 + get_msm_gas(k, 22500, &G2_MSM_DISCOUNT, 524);
		if let Err(e) = gasometer.record_gas(U256::from(gas_cost)) {
			return (Err(e), Vec::new());
		}
		let mut points = Vec::with_capacity(k);
		let mut scalars = Vec::with_capacity(k);
		for i in 0..k {
			let point_data = &input[i * 288..i * 288 + 256];
			let scalar_data = &input[i * 288 + 256..i * 288 + 288];
			let p = match read_g2_point(point_data) {
				Ok(p) => p,
				Err(e) => return (Err(e), Vec::new()),
			};
			if !p.is_in_correct_subgroup_assuming_on_curve() {
				return (Err(ExitException::Other("point not in correct subgroup".into()).into()), Vec::new());
			}
			let s = read_scalar(scalar_data);
			points.push(p);
			scalars.push(s);
		}
		let res = G2Projective::msm_unchecked(&points, &scalars).into_affine();
		(ExitSucceed::Returned.into(), encode_g2_point(res))
	}
}

pub struct Bls12381Pairing;
impl<G: GasMutState> PurePrecompile<G> for Bls12381Pairing {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let k = input.len() / 384;
		if k == 0 {
			return (Err(ExitException::Other("invalid input length".into()).into()), Vec::new());
		}
		let gas_cost = 34000 * (k as u64) + 45000;
		if let Err(e) = gasometer.record_gas(U256::from(gas_cost)) {
			return (Err(e), Vec::new());
		}
		let mut g1_points = Vec::with_capacity(k);
		let mut g2_points = Vec::with_capacity(k);
		for i in 0..k {
			let g1_data = &input[i * 384..i * 384 + 128];
			let g2_data = &input[i * 384 + 128..i * 384 + 384];
			let p1 = match read_g1_point(g1_data) {
				Ok(p) => p,
				Err(e) => return (Err(e), Vec::new()),
			};
			if !p1.is_in_correct_subgroup_assuming_on_curve() {
				return (Err(ExitException::Other("g1 point not in correct subgroup".into()).into()), Vec::new());
			}
			let p2 = match read_g2_point(g2_data) {
				Ok(p) => p,
				Err(e) => return (Err(e), Vec::new()),
			};
			if !p2.is_in_correct_subgroup_assuming_on_curve() {
				return (Err(ExitException::Other("g2 point not in correct subgroup".into()).into()), Vec::new());
			}
			g1_points.push(p1);
			g2_points.push(p2);
		}
		let res = Bls12_381::multi_pairing(g1_points, g2_points);
		let success = res.0.is_one();
		let mut out = vec![0u8; 32];
		if success {
			out[31] = 1;
		}
		(ExitSucceed::Returned.into(), out)
	}
}

pub struct Bls12381MapG1;
impl<G: GasMutState> PurePrecompile<G> for Bls12381MapG1 {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		if let Err(e) = gasometer.record_gas(U256::from(5500u64)) {
			return (Err(e), Vec::new());
		}
		let fp = match read_fp(input) {
			Ok(fp) => fp,
			Err(e) => return (Err(e), Vec::new()),
		};
		let res = WBMap::map_to_curve(fp)
			.expect("map_to_curve is infallible")
			.clear_cofactor();
		(ExitSucceed::Returned.into(), encode_g1_point(res))
	}
}

pub struct Bls12381MapG2;
impl<G: GasMutState> PurePrecompile<G> for Bls12381MapG2 {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		if let Err(e) = gasometer.record_gas(U256::from(110000u64)) {
			return (Err(e), Vec::new());
		}
		let fp2 = match read_fp2(input) {
			Ok(fp2) => fp2,
			Err(e) => return (Err(e), Vec::new()),
		};
		let res = WBMap::map_to_curve(fp2)
			.expect("map_to_curve is infallible")
			.clear_cofactor();
		(ExitSucceed::Returned.into(), encode_g2_point(res))
	}
}
