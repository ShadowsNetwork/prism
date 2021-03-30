#![cfg_attr(not(feature = "std"), no_std)]

/// Current version of pallet Ethereum's storage schema is stored under this key.
pub const PALLET_ETHEREUM_SCHEMA: &'static [u8] = b":ethereum_schema";
