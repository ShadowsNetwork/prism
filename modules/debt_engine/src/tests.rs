//! Unit tests for the debt engine module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, traits::OnFinalize};
use mock::*;
use orml_traits::MultiCurrency;
use sp_runtime::traits::BadOrigin;

#[test]
fn is_debt_unsafe_work() {
	fn is_user_safe(currency_id: CurrencyId, who: &AccountId) -> bool {
		let Position { collateral, debit } = LendModule::positions(currency_id, &who);
		DEBTEngineModule::is_debt_unsafe(currency_id, collateral, debit)
	}

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(is_user_safe(DOS, &ALICE), false);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 50));
		assert_eq!(is_user_safe(DOS, &ALICE), false);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_eq!(is_user_safe(DOS, &ALICE), true);
	});
}

#[test]
fn get_debit_exchange_rate_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			DEBTEngineModule::get_debit_exchange_rate(DOS),
			DefaultDebitExchangeRate::get()
		);
	});
}

#[test]
fn get_liquidation_penalty_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			DEBTEngineModule::get_liquidation_penalty(DOS),
			DefaultLiquidationPenalty::get()
		);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			DEBTEngineModule::get_liquidation_penalty(DOS),
			Rate::saturating_from_rational(2, 10)
		);
	});
}

#[test]
fn get_liquidation_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			DEBTEngineModule::get_liquidation_ratio(DOS),
			DefaultLiquidationRatio::get()
		);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			DEBTEngineModule::get_liquidation_ratio(DOS),
			Ratio::saturating_from_rational(5, 2)
		);
	});
}

#[test]
fn set_global_params_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			DEBTEngineModule::set_global_params(Origin::signed(5), Rate::saturating_from_rational(1, 10000)),
			BadOrigin
		);
		assert_ok!(DEBTEngineModule::set_global_params(
			Origin::signed(1),
			Rate::saturating_from_rational(1, 10000),
		));

		let update_global_stability_fee_event = TestEvent::debt_engine(RawEvent::GlobalStabilityFeeUpdated(
			Rate::saturating_from_rational(1, 10000),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_global_stability_fee_event));

		assert_eq!(
			DEBTEngineModule::global_stability_fee(),
			Rate::saturating_from_rational(1, 10000)
		);
	});
}

#[test]
fn set_collateral_params_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			DEBTEngineModule::set_collateral_params(
				Origin::signed(1),
				LDOT,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
			),
			Error::<Runtime>::InvalidCollateralType
		);

		System::set_block_number(1);
		assert_noop!(
			DEBTEngineModule::set_collateral_params(
				Origin::signed(5),
				DOS,
				Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
				Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
				Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
				Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
				Change::NewValue(10000),
			),
			BadOrigin
		);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let update_stability_fee_event = TestEvent::debt_engine(RawEvent::StabilityFeeUpdated(
			DOS,
			Some(Rate::saturating_from_rational(1, 100000)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_stability_fee_event));
		let update_liquidation_ratio_event = TestEvent::debt_engine(RawEvent::LiquidationRatioUpdated(
			DOS,
			Some(Ratio::saturating_from_rational(3, 2)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_liquidation_ratio_event));
		let update_liquidation_penalty_event = TestEvent::debt_engine(RawEvent::LiquidationPenaltyUpdated(
			DOS,
			Some(Rate::saturating_from_rational(2, 10)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_liquidation_penalty_event));
		let update_required_collateral_ratio_event = TestEvent::debt_engine(RawEvent::RequiredCollateralRatioUpdated(
			DOS,
			Some(Ratio::saturating_from_rational(9, 5)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_required_collateral_ratio_event));
		let update_maximum_total_debit_value_event =
			TestEvent::debt_engine(RawEvent::MaximumTotalDebitValueUpdated(DOS, 10000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_maximum_total_debit_value_event));

		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let new_collateral_params = DEBTEngineModule::collateral_params(DOS);

		assert_eq!(
			new_collateral_params.stability_fee,
			Some(Rate::saturating_from_rational(1, 100000))
		);
		assert_eq!(
			new_collateral_params.liquidation_ratio,
			Some(Ratio::saturating_from_rational(3, 2))
		);
		assert_eq!(
			new_collateral_params.liquidation_penalty,
			Some(Rate::saturating_from_rational(2, 10))
		);
		assert_eq!(
			new_collateral_params.required_collateral_ratio,
			Some(Ratio::saturating_from_rational(9, 5))
		);
		assert_eq!(new_collateral_params.maximum_total_debit_value, 10000);
	});
}

#[test]
fn calculate_collateral_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			DEBTEngineModule::calculate_collateral_ratio(DOS, 100, 50, Price::saturating_from_rational(1, 1)),
			Ratio::saturating_from_rational(100, 50)
		);
	});
}

#[test]
fn check_debit_cap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEBTEngineModule::check_debit_cap(DOS, 9999));
		assert_noop!(
			DEBTEngineModule::check_debit_cap(DOS, 10001),
			Error::<Runtime>::ExceedDebitValueHardCap,
		);
	});
}

#[test]
fn check_position_valid_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));

		MockPriceSource::set_relative_price(None);
		assert_noop!(
			DEBTEngineModule::check_position_valid(DOS, 100, 50),
			Error::<Runtime>::InvalidFeedPrice
		);
		MockPriceSource::set_relative_price(Some(Price::one()));

		assert_ok!(DEBTEngineModule::check_position_valid(DOS, 100, 50));
	});
}

#[test]
fn check_position_valid_failed_when_remain_debit_value_too_small() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));
		assert_noop!(
			DEBTEngineModule::check_position_valid(DOS, 2, 1),
			Error::<Runtime>::RemainDebitValueTooSmall,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_liquidate_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(10, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			DEBTEngineModule::check_position_valid(DOS, 91, 50),
			Error::<Runtime>::BelowLiquidationRatio,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_required_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			DEBTEngineModule::check_position_valid(DOS, 89, 50),
			Error::<Runtime>::BelowRequiredCollateralRatio
		);
	});
}

#[test]
fn adjust_position_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 50),
			Error::<Runtime>::InvalidCollateralType,
		);
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 0);
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 0);
		assert_eq!(LendModule::positions(DOS, ALICE).collateral, 0);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 50));
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 50);
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 50);
		assert_eq!(LendModule::positions(DOS, ALICE).collateral, 100);
		assert_eq!(DEBTEngineModule::adjust_position(&ALICE, DOS, 0, 20).is_ok(), false);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 0, -20));
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 30);
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 30);
		assert_eq!(LendModule::positions(DOS, ALICE).collateral, 100);
	});
}

#[test]
fn remain_debit_value_too_small_check() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 50));
		assert_eq!(DEBTEngineModule::adjust_position(&ALICE, DOS, 0, -49).is_ok(), false);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, -100, -50));
	});
}

#[test]
fn liquidate_unsafe_debt_by_collateral_auction() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 50));
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 50);
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 50);
		assert_eq!(LendModule::positions(DOS, ALICE).collateral, 100);
		assert_noop!(
			DEBTEngineModule::liquidate_unsafe_debt(ALICE, DOS),
			Error::<Runtime>::MustBeUnsafe,
		);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_ok!(DEBTEngineModule::liquidate_unsafe_debt(ALICE, DOS));

		let liquidate_unsafe_debt_event = TestEvent::debt_engine(RawEvent::LiquidateUnsafeDEBT(
			DOS,
			ALICE,
			100,
			50,
			LiquidationStrategy::Auction,
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == liquidate_unsafe_debt_event));

		assert_eq!(DEBTTreasuryModule::debit_pool(), 50);
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 50);
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 0);
		assert_eq!(LendModule::positions(DOS, ALICE).collateral, 0);

		mock_shutdown();
		assert_noop!(
			DEBTEngineModule::liquidate(Origin::none(), DOS, ALICE),
			Error::<Runtime>::AlreadyShutdown
		);
	});
}

#[test]
fn on_finalize_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOT,
			Change::NewValue(Some(Rate::saturating_from_rational(2, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		DEBTEngineModule::on_finalize(1);
		assert_eq!(DEBTEngineModule::debit_exchange_rate(DOS), None);
		assert_eq!(DEBTEngineModule::debit_exchange_rate(DOT), None);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 30));
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 30);
		DEBTEngineModule::on_finalize(2);
		assert_eq!(
			DEBTEngineModule::debit_exchange_rate(DOS),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
		assert_eq!(DEBTEngineModule::debit_exchange_rate(DOT), None);
		DEBTEngineModule::on_finalize(3);
		assert_eq!(
			DEBTEngineModule::debit_exchange_rate(DOS),
			Some(ExchangeRate::saturating_from_rational(10201, 10000))
		);
		assert_eq!(DEBTEngineModule::debit_exchange_rate(DOT), None);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 0, -30));
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(Currencies::free_balance(AUSD, &ALICE), 0);
		DEBTEngineModule::on_finalize(4);
		assert_eq!(
			DEBTEngineModule::debit_exchange_rate(DOS),
			Some(ExchangeRate::saturating_from_rational(10201, 10000))
		);
		assert_eq!(DEBTEngineModule::debit_exchange_rate(DOT), None);
	});
}

#[test]
fn on_emergency_shutdown_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 30));
		DEBTEngineModule::on_finalize(1);
		assert_eq!(
			DEBTEngineModule::debit_exchange_rate(DOS),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
		mock_shutdown();
		assert_eq!(<Runtime as Trait>::EmergencyShutdown::is_shutdown(), true);
		DEBTEngineModule::on_finalize(2);
		assert_eq!(
			DEBTEngineModule::debit_exchange_rate(DOS),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
	});
}

#[test]
fn settle_debt_has_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(DEBTEngineModule::set_collateral_params(
			Origin::signed(1),
			DOS,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 100, 0));
		assert_eq!(Currencies::free_balance(DOS, &ALICE), 900);
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 0);
		assert_eq!(LendModule::positions(DOS, ALICE).collateral, 100);
		assert_noop!(
			DEBTEngineModule::settle_debt_has_debit(ALICE, DOS),
			Error::<Runtime>::NoDebitValue,
		);
		assert_ok!(DEBTEngineModule::adjust_position(&ALICE, DOS, 0, 50));
		assert_eq!(LendModule::positions(DOS, ALICE).debit, 50);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
		assert_eq!(DEBTTreasuryModule::total_collaterals(DOS), 0);
		assert_ok!(DEBTEngineModule::settle_debt_has_debit(ALICE, DOS));

		let settle_debt_in_debit_event = TestEvent::debt_engine(RawEvent::SettleDEBTInDebit(DOS, ALICE));
		assert!(System::events()
			.iter()
			.any(|record| record.event == settle_debt_in_debit_event));

		assert_eq!(LendModule::positions(DOS, ALICE).debit, 0);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 50);
		assert_eq!(DEBTTreasuryModule::total_collaterals(DOS), 50);

		assert_noop!(
			DEBTEngineModule::settle(Origin::none(), DOS, ALICE),
			Error::<Runtime>::MustAfterShutdown
		);
	});
}
