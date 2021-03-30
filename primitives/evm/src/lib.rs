#![cfg_attr(not(feature = "std"), no_std)]

mod precompile;

use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_std::vec::Vec;
use sp_core::{U256, H160};
use evm::ExitReason;

pub use evm::backend::{Basic as Account, Log};
pub use precompile::{Precompile, PrecompileSet, LinearCostPrecompile};

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
/// External input from the transaction.
pub struct Vicinity {
	/// Current transaction gas price.
	pub gas_price: U256,
	/// Origin of the transaction.
	pub origin: H160,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct ExecutionInfo<T> {
	pub exit_reason: ExitReason,
	pub value: T,
	pub used_gas: U256,
	pub logs: Vec<Log>,
}

pub type CallInfo = ExecutionInfo<Vec<u8>>;
pub type CreateInfo = ExecutionInfo<H160>;

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum CallOrCreateInfo {
	Call(CallInfo),
	Create(CreateInfo),
}
