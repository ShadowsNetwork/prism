// Copyright 2019-2020 ShadowsNetwork Inc.
// This file is part of Shadows.

// Shadows is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Shadows is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Shadows.  If not, see <http://www.gnu.org/licenses/>.

//! Minimal Pallet that stores the numeric Ethereum-style chain id in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, traits::Get};

/// Configuration trait of this pallet.
pub trait Config: frame_system::Config {}

impl<T: Config> Get<u64> for Module<T> {
	fn get() -> u64 {
		Self::chain_id()
	}
}

decl_storage! {
	trait Store for Module<T: Config> as ShadowsChainId {
		ChainId get(fn chain_id) config(): u64 = 43;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {}
}
