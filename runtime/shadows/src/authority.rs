//! An orml_authority trait implementation.

use crate::{
	AccountId, AccountIdConversion, AuthoritysOriginId, BadOrigin, BlockNumber, DSWFModuleId, DispatchResult,
	EnsureRoot, EnsureRootOrHalfGeneralCouncil, EnsureRootOrHalfMintxCouncil, EnsureRootOrHalfStake_EarningCouncil,
	EnsureRootOrOneThirdsTechnicalCommittee, EnsureRootOrThreeFourthsGeneralCouncil,
	EnsureRootOrTwoThirdsTechnicalCommittee, MintxTreasuryModuleId, OneDay, Origin, OriginCaller, SevenDays,
	ShadowTreasuryModuleId, Stake_EarningTreasuryModuleId, ZeroDay, HOURS,
};
pub use frame_support::traits::{schedule::Priority, EnsureOrigin, OriginTrait};
use frame_system::ensure_root;
use orml_authority::EnsureDelayed;

pub struct AuthorityConfigImpl;
impl orml_authority::AuthorityConfig<Origin, OriginCaller, BlockNumber> for AuthorityConfigImpl {
	fn check_schedule_dispatch(origin: Origin, _priority: Priority) -> DispatchResult {
		let origin: Result<frame_system::RawOrigin<AccountId>, _> = origin.into();
		match origin {
			Ok(frame_system::RawOrigin::Root) => Ok(()),
			Ok(frame_system::RawOrigin::Signed(caller)) => {
				if caller == ShadowTreasuryModuleId::get().into_account()
					|| caller == MintxTreasuryModuleId::get().into_account()
					|| caller == Stake_EarningTreasuryModuleId::get().into_account()
					|| caller == DSWFModuleId::get().into_account()
				{
					Ok(())
				} else {
					Err(BadOrigin.into())
				}
			}
			_ => Err(BadOrigin.into()),
		}
	}

	fn check_fast_track_schedule(
		origin: Origin,
		_initial_origin: &OriginCaller,
		new_delay: BlockNumber,
	) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			if new_delay / HOURS < 12 {
				EnsureRootOrTwoThirdsTechnicalCommittee::ensure_origin(origin)
					.map_or_else(|e| Err(e.into()), |_| Ok(()))
			} else {
				EnsureRootOrOneThirdsTechnicalCommittee::ensure_origin(origin)
					.map_or_else(|e| Err(e.into()), |_| Ok(()))
			}
		})
	}

	fn check_delay_schedule(origin: Origin, _initial_origin: &OriginCaller) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			EnsureRootOrOneThirdsTechnicalCommittee::ensure_origin(origin).map_or_else(|e| Err(e.into()), |_| Ok(()))
		})
	}

	fn check_cancel_schedule(origin: Origin, initial_origin: &OriginCaller) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			if origin.caller() == initial_origin
				|| EnsureRootOrThreeFourthsGeneralCouncil::ensure_origin(origin).is_ok()
			{
				Ok(())
			} else {
				Err(BadOrigin.into())
			}
		})
	}
}

impl orml_authority::AsOriginId<Origin, OriginCaller> for AuthoritysOriginId {
	fn into_origin(self) -> OriginCaller {
		match self {
			AuthoritysOriginId::Root => Origin::root().caller().clone(),
			AuthoritysOriginId::ShadowTreasury => Origin::signed(ShadowTreasuryModuleId::get().into_account())
				.caller()
				.clone(),
			AuthoritysOriginId::MintxTreasury => Origin::signed(MintxTreasuryModuleId::get().into_account())
				.caller()
				.clone(),
			AuthoritysOriginId::Stake_EarningTreasury => {
				Origin::signed(Stake_EarningTreasuryModuleId::get().into_account())
					.caller()
					.clone()
			}
			AuthoritysOriginId::DSWF => Origin::signed(DSWFModuleId::get().into_account()).caller().clone(),
		}
	}

	fn check_dispatch_from(&self, origin: Origin) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			match self {
			AuthoritysOriginId::Root => <EnsureDelayed<
				SevenDays,
				EnsureRootOrThreeFourthsGeneralCouncil,
				BlockNumber,
				OriginCaller,
			> as EnsureOrigin<Origin>>::ensure_origin(origin)
			.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(())),
			AuthoritysOriginId::ShadowTreasury => {
				<EnsureDelayed<OneDay, EnsureRootOrHalfGeneralCouncil, BlockNumber, OriginCaller> as EnsureOrigin<
					Origin,
				>>::ensure_origin(origin)
				.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
			}
			AuthoritysOriginId::MintxTreasury => {
				<EnsureDelayed<OneDay, EnsureRootOrHalfMintxCouncil, BlockNumber, OriginCaller> as EnsureOrigin<
					Origin,
				>>::ensure_origin(origin)
				.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
			}
			AuthoritysOriginId::Stake_EarningTreasury => {
				<EnsureDelayed<OneDay, EnsureRootOrHalfStake_EarningCouncil, BlockNumber, OriginCaller> as EnsureOrigin<
					Origin,
				>>::ensure_origin(origin)
				.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
			}
			AuthoritysOriginId::DSWF => {
				<EnsureDelayed<ZeroDay, EnsureRoot<AccountId>, BlockNumber, OriginCaller> as EnsureOrigin<
						Origin,
					>>::ensure_origin(origin)
					.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
			}
		}
		})
	}
}
