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

pub use frame_support::traits::KeyOwnerProofSystem;
pub use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_core::crypto::{KeyTypeId, Public};
pub use sp_runtime::traits::NumberFor;

#[macro_export]
macro_rules! runtime_standalone {
	() => {

		/// This runtime version.
		pub const VERSION: RuntimeVersion = RuntimeVersion {
			spec_name: create_runtime_str!("ombre"),
			impl_name: create_runtime_str!("shadows-standalone"),
			authoring_version: 3,
			spec_version: 11,
			impl_version: 1,
			apis: RUNTIME_API_VERSIONS,
			transaction_version: 2,
		};

		impl pallet_aura::Config for Runtime {
			type AuthorityId = AuraId;
		}

		impl pallet_grandpa::Config for Runtime {
			type Event = Event;
			type Call = Call;

			type KeyOwnerProofSystem = ();

			type KeyOwnerProof =
				<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

			type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
				KeyTypeId,
				GrandpaId,
			)>>::IdentificationTuple;

			type HandleEquivocation = ();
			type WeightInfo = ();
		}

		impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
			fn find_author<'a, I>(digests: I) -> Option<H160>
			where
				I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
			{
				if let Some(author_index) = F::find_author(digests) {
					let authority_id = Aura::authorities()[author_index as usize].clone();
					return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
				}
				None
			}
		}

		construct_runtime!(
			pub enum Runtime where
				Block = Block,
				NodeBlock = opaque::Block,
				UncheckedExtrinsic = UncheckedExtrinsic
			{
				System: frame_system::{Module, Call, Config, Storage, Event<T>},
				RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
				Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
				Aura: pallet_aura::{Module, Config<T>, Inherent},
				Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
				Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
				TransactionPayment: pallet_transaction_payment::{Module, Storage},
				Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
				EthereumChainId: pallet_ethereum_chain_id::{Module, Storage, Config},
				Ethereum: pallet_ethereum::{Module, Call, Storage, Event, Config, ValidateUnsigned},
				EVM: pallet_evm::{Module, Config, Call, Storage, Event<T>},
				Stake: stake::{Module, Call, Storage, Event<T>, Config<T>},
				AuthorInherent: author_inherent::{Module, Call, Storage, Inherent, Event<T>},
			}
		);
	};
}
