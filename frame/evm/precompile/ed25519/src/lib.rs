#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::convert::TryFrom;
use fp_evm::LinearCostPrecompile;
use evm::{ExitSucceed, ExitError};
use ed25519_dalek::{PublicKey, Verifier, Signature};

pub struct Ed25519Verify;

impl LinearCostPrecompile for Ed25519Verify {
	const BASE: u64 = 15;
	const WORD: u64 = 3;

	fn execute(
		input: &[u8],
		_: u64,
	) -> core::result::Result<(ExitSucceed, Vec<u8>), ExitError> {
		if input.len() < 128 {
			return Err(ExitError::Other("input must contain 128 bytes".into()));
		};

		let mut i = [0u8; 128];
		i[..128].copy_from_slice(&input[..128]);

		let mut buf = [0u8; 4];

		let msg = &i[0..32];
		let pk = PublicKey::from_bytes(&i[32..64])
			.map_err(|_| ExitError::Other("Public key recover failed".into()))?;
		let sig = Signature::try_from(&i[64..128])
			.map_err(|_| ExitError::Other("Signature recover failed".into()))?;

		// https://docs.rs/rust-crypto/0.2.36/crypto/ed25519/fn.verify.html
		if pk.verify(msg, &sig).is_ok() {
			buf[3] = 0u8;
		} else {
			buf[3] = 1u8;
		};

		Ok((ExitSucceed::Returned, buf.to_vec()))
	}
}
