//! # DEBT Treasury Module
//!
//! ## Overview
//!
//! DEBT Treasury manages the accumulated interest and bad debts generated by
//! DEBTs, and handle excessive surplus or debits timely in order to keep the
//! system healthy with low risk. It's the only entry for issuing/burning stable
//! coin for whole system.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{EnsureOrigin, Get},
	weights::{DispatchClass, Weight},
};
use frame_system::{self as system};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use orml_utilities::with_transaction_result;
use primitives::{Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError, DispatchResult, FixedPointNumber, ModuleId,
};
use support::{AuctionManager, DEBTTreasury, DEBTTreasuryExtended, EXCHANGEManager, Ratio};

mod benchmarking;
mod default_weight;
mod mock;
mod tests;

pub trait WeightInfo {
	fn auction_surplus() -> Weight;
	fn auction_debit() -> Weight;
	fn auction_collateral() -> Weight;
	fn set_collateral_auction_maximum_size() -> Weight;
}

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;

	/// The origin which may update parameters and handle
	/// surplus/debit/collateral. Root can always do this.
	type UpdateOrigin: EnsureOrigin<Self::Origin>;

	/// The Currency for managing assets related to DEBT
	type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

	/// Stablecoin currency id
	type GetStableCurrencyId: Get<CurrencyId>;

	/// Auction manager creates different types of auction to handle system
	/// surplus and debit, and confiscated collateral assets
	type AuctionManagerHandler: AuctionManager<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

	/// Exchange manager is used to swap confiscated collateral assets to stable
	/// currency
	type EXCHANGE: EXCHANGEManager<Self::AccountId, CurrencyId, Balance>;

	/// The cap of lots number when create collateral auction on a liquidation
	/// or to create debit/surplus auction on block end.
	/// If set to 0, does not work.
	type MaxAuctionsCount: Get<u32>;

	/// The DEBT treasury's module id, keep surplus and collateral assets from
	/// liquidation.
	type ModuleId: Get<ModuleId>;

	/// Weight information for the extrinsics in this module.
	type WeightInfo: WeightInfo;
}

decl_event!(
	pub enum Event {
		/// The fixed size for collateral auction under specific collateral type
		/// updated. \[collateral_type, new_size\]
		CollateralAuctionMaximumSizeUpdated(CurrencyId, Balance),
	}
);

decl_error! {
	/// Error for debt treasury module.
	pub enum Error for Module<T: Trait> {
		/// The collateral amount of DEBT treasury is not enough
		CollateralNotEnough,
		/// The surplus pool of DEBT treasury is not enough
		SurplusPoolNotEnough,
		/// debit pool overflow
		DebitPoolOverflow,
		/// The debit pool of DEBT treasury is not enough
		DebitPoolNotEnough,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as DEBTTreasury {
		/// The maximum amount of collateral amount for sale per collateral auction
		pub CollateralAuctionMaximumSize get(fn collateral_auction_maximum_size): map hasher(twox_64_concat) CurrencyId => Balance;

		/// Current total debit value of system. It's not same as debit in DEBT engine,
		/// it is the bad debt of the system.
		pub DebitPool get(fn debit_pool): Balance;
	}

	add_extra_genesis {
		config(collateral_auction_maximum_size): Vec<(CurrencyId, Balance)>;

		build(|config: &GenesisConfig| {
			config.collateral_auction_maximum_size.iter().for_each(|(currency_id, size)| {
				CollateralAuctionMaximumSize::insert(currency_id, size);
			});
		})
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		/// Stablecoin currency id
		const GetStableCurrencyId: CurrencyId = T::GetStableCurrencyId::get();

		/// Lots cap when create auction
		const MaxAuctionsCount: u32 = T::MaxAuctionsCount::get();

		/// The DEBT treasury's module id, keep surplus and collateral assets from liquidation.
		const ModuleId: ModuleId = T::ModuleId::get();

		#[weight = T::WeightInfo::auction_surplus()]
		pub fn auction_surplus(origin, amount: Balance) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				ensure!(
					Self::surplus_pool().saturating_sub(T::AuctionManagerHandler::get_total_surplus_in_auction()) >= amount,
					Error::<T>::SurplusPoolNotEnough,
				);
				T::AuctionManagerHandler::new_surplus_auction(amount)
			})?;
		}

		#[weight = T::WeightInfo::auction_debit()]
		pub fn auction_debit(origin, amount: Balance, initial_price: Balance) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				ensure!(
					Self::debit_pool().saturating_sub(T::AuctionManagerHandler::get_total_debit_in_auction()) >= amount,
					Error::<T>::DebitPoolNotEnough,
				);
				T::AuctionManagerHandler::new_debit_auction(amount, initial_price)
			})?;
		}

		#[weight = T::WeightInfo::auction_collateral()]
		pub fn auction_collateral(origin, currency_id: CurrencyId, amount: Balance, target: Balance, splited: bool) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				<Self as DEBTTreasuryExtended<T::AccountId>>::create_collateral_auctions(
					currency_id,
					amount,
					target,
					Self::account_id(),
					splited,
				)
			})?;
		}

		/// Update parameters related to collateral auction under specific collateral type
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: collateral type
		/// - `surplus_buffer_size`: collateral auction maximum size
		///
		/// # <weight>
		/// - Complexity: `O(1)`
		/// - Db reads: 0
		/// - Db writes: 1
		/// -------------------
		/// Base Weight: 24.64 µs
		/// # </weight>
		#[weight = (T::WeightInfo::set_collateral_auction_maximum_size(), DispatchClass::Operational)]
		pub fn set_collateral_auction_maximum_size(origin, currency_id: CurrencyId, size: Balance) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				CollateralAuctionMaximumSize::insert(currency_id, size);
				Self::deposit_event(Event::CollateralAuctionMaximumSizeUpdated(currency_id, size));
				Ok(())
			})?;
		}

		/// Handle excessive surplus or debits of system when block end
		fn on_finalize(_now: T::BlockNumber) {
			// offset the same amount between debit pool and surplus pool
			Self::offset_surplus_and_debit();
		}
	}
}

impl<T: Trait> Module<T> {
	/// Get account of debt treasury module.
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}

	/// Get current total surplus of system.
	pub fn surplus_pool() -> Balance {
		T::Currency::free_balance(T::GetStableCurrencyId::get(), &Self::account_id())
	}

	/// Get total collateral amount of debt treasury module.
	pub fn total_collaterals(currency_id: CurrencyId) -> Balance {
		T::Currency::free_balance(currency_id, &Self::account_id())
	}

	/// Get collateral amount not in auction
	pub fn total_collaterals_not_in_auction(currency_id: CurrencyId) -> Balance {
		T::Currency::free_balance(currency_id, &Self::account_id())
			.saturating_sub(T::AuctionManagerHandler::get_total_collateral_in_auction(currency_id))
	}

	fn offset_surplus_and_debit() {
		let offset_amount = sp_std::cmp::min(Self::debit_pool(), Self::surplus_pool());

		// Burn the amount that is equal to offset amount of stable currency.
		if !offset_amount.is_zero()
			&& T::Currency::withdraw(T::GetStableCurrencyId::get(), &Self::account_id(), offset_amount).is_ok()
		{
			DebitPool::mutate(|debit| {
				*debit = debit
					.checked_sub(offset_amount)
					.expect("offset = min(debit, surplus); qed")
			});
		}
	}
}

impl<T: Trait> DEBTTreasury<T::AccountId> for Module<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	fn get_surplus_pool() -> Self::Balance {
		Self::surplus_pool()
	}

	fn get_debit_pool() -> Self::Balance {
		Self::debit_pool()
	}

	fn get_total_collaterals(id: Self::CurrencyId) -> Self::Balance {
		Self::total_collaterals(id)
	}

	fn get_debit_proportion(amount: Self::Balance) -> Ratio {
		let stable_total_supply = T::Currency::total_issuance(T::GetStableCurrencyId::get());
		Ratio::checked_from_rational(amount, stable_total_supply).unwrap_or_default()
	}

	fn on_system_debit(amount: Self::Balance) -> DispatchResult {
		DebitPool::try_mutate(|debit_pool| -> DispatchResult {
			*debit_pool = debit_pool.checked_add(amount).ok_or(Error::<T>::DebitPoolOverflow)?;
			Ok(())
		})
	}

	fn on_system_surplus(amount: Self::Balance) -> DispatchResult {
		Self::issue_debit(&Self::account_id(), amount, true)
	}

	fn issue_debit(who: &T::AccountId, debit: Self::Balance, backed: bool) -> DispatchResult {
		// increase system debit if the debit is unbacked
		if !backed {
			Self::on_system_debit(debit)?;
		}
		T::Currency::deposit(T::GetStableCurrencyId::get(), who, debit)?;

		Ok(())
	}

	fn burn_debit(who: &T::AccountId, debit: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::GetStableCurrencyId::get(), who, debit)
	}

	fn deposit_surplus(from: &T::AccountId, surplus: Self::Balance) -> DispatchResult {
		T::Currency::transfer(T::GetStableCurrencyId::get(), from, &Self::account_id(), surplus)
	}

	fn deposit_collateral(from: &T::AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(currency_id, from, &Self::account_id(), amount)
	}

	fn withdraw_collateral(to: &T::AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(currency_id, &Self::account_id(), to, amount)
	}
}

impl<T: Trait> DEBTTreasuryExtended<T::AccountId> for Module<T> {
	/// Swap exact amount of collateral in auction to stable,
	/// return actual target stable amount
	fn swap_exact_collateral_in_auction_to_stable(
		currency_id: CurrencyId,
		supply_amount: Balance,
		min_target_amount: Balance,
		price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		ensure!(
			Self::total_collaterals(currency_id) >= supply_amount
				&& T::AuctionManagerHandler::get_total_collateral_in_auction(currency_id) >= supply_amount,
			Error::<T>::CollateralNotEnough,
		);

		T::EXCHANGE::swap_with_exact_supply(
			&Self::account_id(),
			&[currency_id, T::GetStableCurrencyId::get()],
			supply_amount,
			min_target_amount,
			price_impact_limit,
		)
	}

	/// swap collateral which not in auction to get exact stable,
	/// return actual supply collateral amount
	fn swap_collateral_not_in_auction_with_exact_stable(
		currency_id: CurrencyId,
		target_amount: Balance,
		max_supply_amount: Balance,
		price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		ensure!(
			Self::total_collaterals_not_in_auction(currency_id) >= max_supply_amount,
			Error::<T>::CollateralNotEnough,
		);

		T::EXCHANGE::swap_with_exact_target(
			&Self::account_id(),
			&[currency_id, T::GetStableCurrencyId::get()],
			target_amount,
			max_supply_amount,
			price_impact_limit,
		)
	}

	fn create_collateral_auctions(
		currency_id: CurrencyId,
		amount: Balance,
		target: Balance,
		refund_receiver: T::AccountId,
		splited: bool,
	) -> DispatchResult {
		ensure!(
			Self::total_collaterals_not_in_auction(currency_id) >= amount,
			Error::<T>::CollateralNotEnough,
		);

		let mut unhandled_collateral_amount = amount;
		let mut unhandled_target = target;
		let collateral_auction_maximum_size = Self::collateral_auction_maximum_size(currency_id);
		let max_auctions_count: Balance = T::MaxAuctionsCount::get().into();
		let lots_count = if !splited
			|| max_auctions_count.is_zero()
			|| collateral_auction_maximum_size.is_zero()
			|| amount <= collateral_auction_maximum_size
		{
			One::one()
		} else {
			let mut count = amount
				.checked_div(collateral_auction_maximum_size)
				.expect("collateral auction maximum size is not zero; qed");

			let remainder = amount
				.checked_rem(collateral_auction_maximum_size)
				.expect("collateral auction maximum size is not zero; qed");
			if !remainder.is_zero() {
				count = count.saturating_add(One::one());
			}
			sp_std::cmp::min(count, max_auctions_count)
		};
		let average_amount_per_lot = amount.checked_div(lots_count).expect("lots count is at least 1; qed");
		let average_target_per_lot = target.checked_div(lots_count).expect("lots count is at least 1; qed");
		let mut created_lots: Balance = Zero::zero();

		while !unhandled_collateral_amount.is_zero() {
			created_lots = created_lots.saturating_add(One::one());
			let (lot_collateral_amount, lot_target) = if created_lots == lots_count {
				// the last lot may be have some remnant than average
				(unhandled_collateral_amount, unhandled_target)
			} else {
				(average_amount_per_lot, average_target_per_lot)
			};

			T::AuctionManagerHandler::new_collateral_auction(
				&refund_receiver,
				currency_id,
				lot_collateral_amount,
				lot_target,
			)?;

			unhandled_collateral_amount = unhandled_collateral_amount.saturating_sub(lot_collateral_amount);
			unhandled_target = unhandled_target.saturating_sub(lot_target);
		}
		Ok(())
	}
}