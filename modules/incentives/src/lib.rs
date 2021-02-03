#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{EnsureOrigin, Get, Happened},
	weights::Weight,
	IterableStorageMap,
};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, RewardHandler};
use orml_utilities::with_transaction_result;
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, UniqueSaturatedInto, Zero},
	FixedPointNumber, ModuleId, RuntimeDebug,
};
use sp_std::prelude::*;
use support::{DEBTTreasury, EXCHANGEManager, EmergencyShutdown, Rate};

mod default_weight;
mod mock;
mod tests;

pub trait WeightInfo {
	fn deposit_exchange_share() -> Weight;
	fn withdraw_exchange_share() -> Weight;
	fn claim_rewards() -> Weight;
	fn update_lend_incentive_rewards(c: u32) -> Weight;
	fn update_exchange_incentive_rewards(c: u32) -> Weight;
	fn update_stake_earning_incentive_reward() -> Weight;
	fn update_exchange_saving_rates(c: u32) -> Weight;
}

/// PoolId for various rewards pools
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PoolId {
	/// Rewards(DOS) pool for users who open CDP
	Lend(CurrencyId),
	/// Rewards(DOS) pool for market makers who provide exchange liquidity
	ExchangeIncentive(CurrencyId),
	/// Rewards(XUSD) pool for liquidators who provide exchange liquidity to
	/// participate automatic liquidation
	ExchangeSaving(CurrencyId),
	/// Rewards(DOS) pool for users who staking by StakeEarning protocol
	StakeEarning,
}

decl_error! {
	/// Error for incentives module.
	pub enum Error for Module<T: Trait> {
		/// Share amount is not enough
		NotEnough,
		/// Invalid currency id
		InvalidCurrencyId,
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		Balance = Balance,
		CurrencyId = CurrencyId,
	{
		/// Deposit EXCHANGE share. \[who, exchange_share_type, deposit_amount\]
		DepositEXCHANGEShare(AccountId, CurrencyId, Balance),
		/// Withdraw EXCHANGE share. \[who, exchange_share_type, withdraw_amount\]
		WithdrawEXCHANGEShare(AccountId, CurrencyId, Balance),
	}
);

pub trait Trait:
	frame_system::Trait + orml_rewards::Trait<Share = Balance, Balance = Balance, PoolId = PoolId>
{
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The vault account to keep rewards for type LendIncentive PoolId
	type LendIncentivePool: Get<Self::AccountId>;

	/// The vault account to keep rewards for type ExchangeIncentive and
	/// ExchangeSaving PoolId
	type ExchangeIncentivePool: Get<Self::AccountId>;

	/// The vault account to keep rewards for type StakeEarningIncentive PoolId
	type StakeEarningIncentivePool: Get<Self::AccountId>;

	/// The period to accumulate rewards
	type AccumulatePeriod: Get<Self::BlockNumber>;

	/// The incentive reward type (should be DOS)
	type IncentiveCurrencyId: Get<CurrencyId>;

	/// The saving reward type (should be XUSD)
	type SavingCurrencyId: Get<CurrencyId>;

	/// The origin which may update incentive related params
	type UpdateOrigin: EnsureOrigin<Self::Origin>;

	/// CDP treasury to issue rewards in XUSD
	type DEBTTreasury: DEBTTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

	/// Currency for transfer/issue assets
	type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

	/// EXCHANGE to supply liquidity info
	type EXCHANGE: EXCHANGEManager<Self::AccountId, CurrencyId, Balance>;

	/// Emergency shutdown.
	type EmergencyShutdown: EmergencyShutdown;

	/// The module id, keep EXCHANGEShare LP.
	type ModuleId: Get<ModuleId>;

	/// Weight information for the extrinsics in this module.
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Trait> as Incentives {
		/// Mapping from collateral currency type to its lend incentive reward amount per period
		pub LendIncentiveRewards get(fn lend_incentive_rewards): map hasher(twox_64_concat) CurrencyId => Balance;

		/// Mapping from exchange liquidity currency type to its lend incentive reward amount per period
		pub EXCHANGEIncentiveRewards get(fn exchange_incentive_rewards): map hasher(twox_64_concat) CurrencyId => Balance;

		/// StakeEarning incentive reward amount
		pub StakeEarningIncentiveReward get(fn stake_earning_incentive_reward): Balance;

		/// Mapping from exchange liquidity currency type to its saving rate
		pub EXCHANGESavingRates get(fn exchange_saving_rates): map hasher(twox_64_concat) CurrencyId => Rate;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// The vault account to keep rewards for type LendIncentive PoolId
		const LendIncentivePool: T::AccountId = T::LendIncentivePool::get();

		/// The vault account to keep rewards for type ExchangeIncentive and ExchangeSaving PoolId
		const ExchangeIncentivePool: T::AccountId = T::ExchangeIncentivePool::get();

		/// The vault account to keep rewards for type StakeEarningIncentive PoolId
		const StakeEarningIncentivePool: T::AccountId = T::StakeEarningIncentivePool::get();

		/// The period to accumulate rewards
		const AccumulatePeriod: T::BlockNumber = T::AccumulatePeriod::get();

		/// The incentive reward type (should be DOS)
		const IncentiveCurrencyId: CurrencyId = T::IncentiveCurrencyId::get();

		/// The saving reward type (should be XUSD)
		const SavingCurrencyId: CurrencyId = T::SavingCurrencyId::get();

		#[weight = <T as Trait>::WeightInfo::deposit_exchange_share()]
		pub fn deposit_exchange_share(origin, lp_currency_id: CurrencyId, amount: Balance) {
			with_transaction_result(|| {
				let who = ensure_signed(origin)?;

				match lp_currency_id {
					CurrencyId::EXCHANGEShare(_, _) => {},
					_ => return Err(Error::<T>::InvalidCurrencyId.into()),
				}

				T::Currency::transfer(lp_currency_id, &who, &Self::account_id(), amount)?;
				OnAddLiquidity::<T>::happened(&(who.clone(), lp_currency_id, amount.unique_saturated_into()));

				Self::deposit_event(RawEvent::DepositEXCHANGEShare(
					who,
					lp_currency_id,
					amount,
				));
				Ok(())
			})?;
		}

		#[weight = <T as Trait>::WeightInfo::withdraw_exchange_share()]
		pub fn withdraw_exchange_share(origin, lp_currency_id: CurrencyId, amount: Balance) {
			with_transaction_result(|| {
				let who = ensure_signed(origin)?;

				match lp_currency_id {
					CurrencyId::EXCHANGEShare(_, _) => {},
					_ => return Err(Error::<T>::InvalidCurrencyId.into()),
				}

				ensure!(
					<orml_rewards::Module<T>>::share_and_withdrawn_reward(PoolId::ExchangeIncentive(lp_currency_id), &who).0 >= amount
					&& <orml_rewards::Module<T>>::share_and_withdrawn_reward(PoolId::ExchangeSaving(lp_currency_id), &who).0 >= amount,
					Error::<T>::NotEnough,
				);
				OnRemoveLiquidity::<T>::happened(&(who.clone(), lp_currency_id, amount));
				T::Currency::transfer(lp_currency_id, &Self::account_id(), &who, amount)?;

				Self::deposit_event(RawEvent::WithdrawEXCHANGEShare(
					who,
					lp_currency_id,
					amount,
				));
				Ok(())
			})?;
		}

		#[weight = <T as Trait>::WeightInfo::claim_rewards()]
		pub fn claim_rewards(origin, pool_id: T::PoolId) {
			with_transaction_result(|| {
				let who = ensure_signed(origin)?;
				<orml_rewards::Module<T>>::claim_rewards(&who, pool_id);
				Ok(())
			})?;
		}

		#[weight = <T as Trait>::WeightInfo::update_lend_incentive_rewards(updates.len() as u32)]
		pub fn update_lend_incentive_rewards(
			origin,
			updates: Vec<(CurrencyId, Balance)>,
		) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				for (currency_id, amount) in updates {
					LendIncentiveRewards::insert(currency_id, amount);
				}
				Ok(())
			})?;
		}

		#[weight = <T as Trait>::WeightInfo::update_exchange_incentive_rewards(updates.len() as u32)]
		pub fn update_exchange_incentive_rewards(
			origin,
			updates: Vec<(CurrencyId, Balance)>,
		) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				for (currency_id, amount) in updates {
					match currency_id {
						CurrencyId::EXCHANGEShare(_, _) => {},
						_ => return Err(Error::<T>::InvalidCurrencyId.into()),
					}

					EXCHANGEIncentiveRewards::insert(currency_id, amount);
				}
				Ok(())
			})?;
		}

		#[weight = <T as Trait>::WeightInfo::update_stake_earning_incentive_reward()]
		pub fn update_stake_earning_incentive_reward(
			origin,
			update: Balance,
		) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				StakeEarningIncentiveReward::put(update);
				Ok(())
			})?;
		}

		#[weight = <T as Trait>::WeightInfo::update_exchange_saving_rates(updates.len() as u32)]
		pub fn update_exchange_saving_rates(
			origin,
			updates: Vec<(CurrencyId, Rate)>,
		) {
			with_transaction_result(|| {
				T::UpdateOrigin::ensure_origin(origin)?;
				for (currency_id, rate) in updates {
					match currency_id {
						CurrencyId::EXCHANGEShare(_, _) => {},
						_ => return Err(Error::<T>::InvalidCurrencyId.into()),
					}

					EXCHANGESavingRates::insert(currency_id, rate);
				}
				Ok(())
			})?;
		}
	}
}

pub struct OnAddLiquidity<T>(sp_std::marker::PhantomData<T>);
impl<T: Trait> Happened<(T::AccountId, CurrencyId, Balance)> for OnAddLiquidity<T> {
	fn happened(info: &(T::AccountId, CurrencyId, Balance)) {
		let (who, currency_id, increase_share) = info;
		<orml_rewards::Module<T>>::add_share(who, PoolId::ExchangeIncentive(*currency_id), *increase_share);
		<orml_rewards::Module<T>>::add_share(who, PoolId::ExchangeSaving(*currency_id), *increase_share);
	}
}

pub struct OnRemoveLiquidity<T>(sp_std::marker::PhantomData<T>);
impl<T: Trait> Happened<(T::AccountId, CurrencyId, Balance)> for OnRemoveLiquidity<T> {
	fn happened(info: &(T::AccountId, CurrencyId, Balance)) {
		let (who, currency_id, decrease_share) = info;
		<orml_rewards::Module<T>>::remove_share(who, PoolId::ExchangeIncentive(*currency_id), *decrease_share);
		<orml_rewards::Module<T>>::remove_share(who, PoolId::ExchangeSaving(*currency_id), *decrease_share);
	}
}

pub struct OnUpdateLoan<T>(sp_std::marker::PhantomData<T>);
impl<T: Trait> Happened<(T::AccountId, CurrencyId, Amount, Balance)> for OnUpdateLoan<T> {
	fn happened(info: &(T::AccountId, CurrencyId, Amount, Balance)) {
		let (who, currency_id, adjustment, previous_amount) = info;
		let adjustment_abs =
			sp_std::convert::TryInto::<Balance>::try_into(adjustment.saturating_abs()).unwrap_or_default();

		if !adjustment_abs.is_zero() {
			let new_share_amount = if adjustment.is_positive() {
				previous_amount.saturating_add(adjustment_abs)
			} else {
				previous_amount.saturating_sub(adjustment_abs)
			};

			<orml_rewards::Module<T>>::set_share(who, PoolId::Lend(*currency_id), new_share_amount);
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}
}

impl<T: Trait> RewardHandler<T::AccountId, T::BlockNumber> for Module<T> {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = PoolId;
	type CurrencyId = CurrencyId;

	fn accumulate_reward(now: T::BlockNumber, callback: impl Fn(PoolId, Balance)) -> Vec<(CurrencyId, Balance)> {
		let mut accumulated_rewards: Vec<(CurrencyId, Balance)> = vec![];

		if !T::EmergencyShutdown::is_shutdown() && now % T::AccumulatePeriod::get() == Zero::zero() {
			let mut accumulated_incentive: Balance = Zero::zero();
			let mut accumulated_saving: Balance = Zero::zero();
			let incentive_currency_id = T::IncentiveCurrencyId::get();
			let saving_currency_id = T::SavingCurrencyId::get();

			for (pool_id, pool_info) in orml_rewards::Pools::<T>::iter() {
				if !pool_info.total_shares.is_zero() {
					match pool_id {
						PoolId::Lend(currency_id) => {
							let incentive_reward = Self::lend_incentive_rewards(currency_id);

							// TODO: transfer from RESERVED TREASURY instead of issuing
							if !incentive_reward.is_zero()
								&& T::Currency::deposit(
									incentive_currency_id,
									&T::LendIncentivePool::get(),
									incentive_reward,
								)
								.is_ok()
							{
								callback(pool_id, incentive_reward);
								accumulated_incentive = accumulated_incentive.saturating_add(incentive_reward);
							}
						}

						PoolId::ExchangeIncentive(currency_id) => {
							let incentive_reward = Self::exchange_incentive_rewards(currency_id);

							// TODO: transfer from RESERVED TREASURY instead of issuing
							if !incentive_reward.is_zero()
								&& T::Currency::deposit(
									incentive_currency_id,
									&T::ExchangeIncentivePool::get(),
									incentive_reward,
								)
								.is_ok()
							{
								callback(pool_id, incentive_reward);
								accumulated_incentive = accumulated_incentive.saturating_add(incentive_reward);
							}
						}

						PoolId::ExchangeSaving(currency_id) => {
							let exchange_saving_rate = Self::exchange_saving_rates(currency_id);
							if !exchange_saving_rate.is_zero() {
								if let CurrencyId::EXCHANGEShare(token_symbol_a, token_symbol_b) = currency_id {
									let (currency_id_a, currency_id_b) =
										(CurrencyId::Token(token_symbol_a), CurrencyId::Token(token_symbol_b));

									// accumulate saving reward only for liquidity pool of saving currency id
									let saving_currency_amount = if currency_id_a == saving_currency_id {
										T::EXCHANGE::get_liquidity_pool(saving_currency_id, currency_id_b).0
									} else if currency_id_b == saving_currency_id {
										T::EXCHANGE::get_liquidity_pool(saving_currency_id, currency_id_a).0
									} else {
										Zero::zero()
									};

									if !saving_currency_amount.is_zero() {
										let saving_reward =
											exchange_saving_rate.saturating_mul_int(saving_currency_amount);
										if T::DEBTTreasury::issue_debit(
											&T::ExchangeIncentivePool::get(),
											saving_reward,
											false,
										)
										.is_ok()
										{
											callback(pool_id, saving_reward);
											accumulated_saving = accumulated_saving.saturating_add(saving_reward);
										}
									}
								}
							}
						}

						PoolId::StakeEarning => {
							let incentive_reward = Self::stake_earning_incentive_reward();

							// TODO: transfer from RESERVED TREASURY instead of issuing
							if !incentive_reward.is_zero()
								&& T::Currency::deposit(
									incentive_currency_id,
									&T::StakeEarningIncentivePool::get(),
									incentive_reward,
								)
								.is_ok()
							{
								callback(pool_id, incentive_reward);
								accumulated_incentive = accumulated_incentive.saturating_add(incentive_reward);
							}
						}
					}
				}
			}

			if !accumulated_incentive.is_zero() {
				accumulated_rewards.push((incentive_currency_id, accumulated_incentive));
			}
			if !accumulated_saving.is_zero() {
				accumulated_rewards.push((saving_currency_id, accumulated_saving));
			}
		}

		accumulated_rewards
	}

	fn payout(who: &T::AccountId, pool_id: PoolId, amount: Balance) {
		let (pool_account, currency_id) = match pool_id {
			PoolId::Lend(_) => (T::LendIncentivePool::get(), T::IncentiveCurrencyId::get()),
			PoolId::ExchangeIncentive(_) => (T::ExchangeIncentivePool::get(), T::IncentiveCurrencyId::get()),
			PoolId::ExchangeSaving(_) => (T::ExchangeIncentivePool::get(), T::SavingCurrencyId::get()),
			PoolId::StakeEarning => (T::StakeEarningIncentivePool::get(), T::IncentiveCurrencyId::get()),
		};

		// payout the reward to user from the pool. it should not affect the
		// process, ignore the result to continue. if it fails, just the user will not
		// be rewarded, there will not increase user balance.
		let _ = T::Currency::transfer(currency_id, &pool_account, &who, amount);
	}
}
