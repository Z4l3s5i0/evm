use evm_interpreter::uint::{H160, H256, U256};
use k256::ecdsa::{Signature, VerifyingKey};
use sha3::{Digest, Keccak256};
use alloc::vec::Vec;

pub fn recover_address(chain_id: U256, address: H160, nonce: U256, v: u8, r: H256, s: H256) -> Option<H160> {
    let mut payload = Vec::new();
    payload.push(0x05); // Magic

    let mut chain_id_bytes = [0u8; 32];
    chain_id.to_big_endian(&mut chain_id_bytes);
    payload.extend_from_slice(&chain_id_bytes);

    payload.extend_from_slice(address.as_bytes());

    let mut nonce_bytes = [0u8; 32];
    nonce.to_big_endian(&mut nonce_bytes);
    payload.extend_from_slice(&nonce_bytes);

    let msg_hash = Keccak256::digest(&payload);

    let mut r_arr = [0u8; 32];
    r_arr.copy_from_slice(r.as_bytes());
    let mut s_arr = [0u8; 32];
    s_arr.copy_from_slice(s.as_bytes());

    let signature = Signature::from_scalars(r_arr, s_arr).ok()?;

    // Recovery ID from v. For EIP-7702, v is 0 or 1.
    let recid = k256::ecdsa::RecoveryId::from_byte(v)?;

    let vk = VerifyingKey::recover_from_prehash(&msg_hash, &signature, recid).ok()?;
    let public_key = vk.to_encoded_point(false);
    let public_key_bytes = public_key.as_bytes();

    // Hash public key (skip the first byte 0x04)
    let hash = Keccak256::digest(&public_key_bytes[1..]);
    let mut res = H160::default();
    res.as_bytes_mut().copy_from_slice(&hash[12..]);
    Some(res)
}
