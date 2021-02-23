use crate::{
	AccountId, Amount, Balance, CollateralCurrencyIds, CurrencyId, DebtEngine, EmergencyShutdown, Exchange,
	GetStableCurrencyId, MaxSlippageSwapWithEXCHANGE, MinimumDebitValue, Price, Rate, Ratio, Runtime, ShadowsOracle,
	TokenSymbol, DOLLARS,
};

use super::utils::set_balance;
use core::convert::TryInto;
use frame_benchmarking::account;
use frame_system::RawOrigin;
use module_support::EXCHANGEManager;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::Change;
use sp_runtime::{traits::UniqueSaturatedInto, FixedPointNumber};
use sp_std::prelude::*;

const SEED: u32 = 0;

fn inject_liquidity(
	maker: AccountId,
	currency_id: CurrencyId,
	max_amount: Balance,
	max_other_currency_amount: Balance,
) -> Result<(), &'static str> {
	let base_currency_id = GetStableCurrencyId::get();

	// set balance
	set_balance(currency_id, &maker, max_other_currency_amount.unique_saturated_into());
	set_balance(base_currency_id, &maker, max_amount.unique_saturated_into());

	Exchange::add_liquidity(
		RawOrigin::Signed(maker.clone()).into(),
		base_currency_id,
		currency_id,
		max_amount,
		max_other_currency_amount,
	)?;

	Ok(())
}

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	DOLLARS.saturating_mul(d)
}

runtime_benchmarks! {
	{ Runtime, module_debt_engine }

	_ {}

	set_collateral_params {
	}: _(
		RawOrigin::Root,
		CurrencyId::Token(TokenSymbol::DOT),
		Change::NewValue(Some(Rate::saturating_from_rational(1, 1000000))),
		Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
		Change::NewValue(Some(Rate::saturating_from_rational(20, 100))),
		Change::NewValue(Some(Ratio::saturating_from_rational(180, 100))),
		Change::NewValue(dollar(100000))
	)

	set_global_params {
	}: _(RawOrigin::Root, Rate::saturating_from_rational(1, 1000000))

	// `liquidate` by_auction
	liquidate_by_auction {
		let owner: AccountId = account("owner", 0, SEED);
		let currency_id: CurrencyId = CollateralCurrencyIds::get()[0];
		let min_debit_value = MinimumDebitValue::get();
		let debit_exchange_rate = DebtEngine::get_debit_exchange_rate(currency_id);
		let collateral_price = Price::one();		// 1 USD
		let min_debit_amount = debit_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_debit_value);
		let min_debit_amount: Amount = min_debit_amount.unique_saturated_into();
		let collateral_amount = (min_debit_value * 2).unique_saturated_into();

		// set balance
		set_balance(currency_id, &owner, collateral_amount);

		// feed price
		ShadowsOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, collateral_price)])?;

		// set risk params
		DebtEngine::set_collateral_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_debit_value * 100),
		)?;

		// adjust position
		DebtEngine::adjust_position(&owner, currency_id, collateral_amount.try_into().unwrap(), min_debit_amount)?;

		// modify liquidation rate to make the debt unsafe
		DebtEngine::set_collateral_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(1000, 100))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		)?;
	}: liquidate(RawOrigin::None, currency_id, owner)

	// `liquidate` by exchange
	liquidate_by_exchange {
		let owner: AccountId = account("owner", 0, SEED);
		let funder: AccountId = account("funder", 0, SEED);
		let currency_id: CurrencyId = CollateralCurrencyIds::get()[0];
		let min_debit_value = MinimumDebitValue::get();
		let debit_exchange_rate = DebtEngine::get_debit_exchange_rate(currency_id);
		let collateral_price = Price::one();		// 1 USD
		let min_debit_amount = debit_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_debit_value);
		let min_debit_amount: Amount = min_debit_amount.unique_saturated_into();
		let collateral_amount = (min_debit_value * 2).unique_saturated_into();
		let base_currency_id = GetStableCurrencyId::get();
		let max_slippage_swap_with_exchange = MaxSlippageSwapWithEXCHANGE::get();
		let collateral_amount_in_exchange = max_slippage_swap_with_exchange.reciprocal().unwrap().saturating_mul_int(min_debit_value * 10);
		let base_amount_in_exchange = collateral_amount_in_exchange * 2;

		inject_liquidity(funder.clone(), currency_id, base_amount_in_exchange, collateral_amount_in_exchange)?;

		// set balance
		set_balance(currency_id, &owner, collateral_amount);

		// feed price
		ShadowsOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, collateral_price)])?;

		// set risk params
		DebtEngine::set_collateral_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_debit_value * 100),
		)?;

		// adjust position
		DebtEngine::adjust_position(&owner, currency_id, collateral_amount.try_into().unwrap(), min_debit_amount)?;

		// modify liquidation rate to make the debt unsafe
		DebtEngine::set_collateral_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(1000, 100))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		)?;
	}: liquidate(RawOrigin::None, currency_id, owner)
	verify {
		let (other_currency_amount, base_currency_amount) = Exchange::get_liquidity_pool(currency_id, base_currency_id);
		assert!(other_currency_amount > collateral_amount_in_exchange);
		assert!(base_currency_amount < base_amount_in_exchange);
	}

	settle {
		let owner: AccountId = account("owner", 0, SEED);
		let currency_id: CurrencyId = CollateralCurrencyIds::get()[0];
		let min_debit_value = MinimumDebitValue::get();
		let debit_exchange_rate = DebtEngine::get_debit_exchange_rate(currency_id);
		let collateral_price = Price::one();		// 1 USD
		let min_debit_amount = debit_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_debit_value);
		let min_debit_amount: Amount = min_debit_amount.unique_saturated_into();
		let collateral_amount = (min_debit_value * 2).unique_saturated_into();

		// set balance
		set_balance(currency_id, &owner, collateral_amount);

		// feed price
		ShadowsOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, collateral_price)])?;

		// set risk params
		DebtEngine::set_collateral_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_debit_value * 100),
		)?;

		// adjust position
		DebtEngine::adjust_position(&owner, currency_id, collateral_amount.try_into().unwrap(), min_debit_amount)?;

		// shutdown
		EmergencyShutdown::emergency_shutdown(RawOrigin::Root.into())?;
	}: _(RawOrigin::None, currency_id, owner)
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into()
	}

	#[test]
	fn test_set_collateral_params() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_collateral_params());
		});
	}

	#[test]
	fn test_set_global_params() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_global_params());
		});
	}

	#[test]
	fn test_liquidate_by_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_liquidate_by_auction());
		});
	}

	#[test]
	fn test_liquidate_by_exchange() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_liquidate_by_exchange());
		});
	}

	#[test]
	fn test_settle() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_settle());
		});
	}
}