//! Unit tests for the debt treasury module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, traits::OnFinalize};
use mock::*;
use sp_runtime::traits::BadOrigin;

#[test]
fn surplus_pool_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_ok!(Currencies::deposit(
			GetStableCurrencyId::get(),
			&DEBTTreasuryModule::account_id(),
			500
		));
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 500);
	});
}

#[test]
fn total_collaterals_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 0);
		assert_ok!(Currencies::deposit(BTC, &DEBTTreasuryModule::account_id(), 10));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 10);
	});
}

#[test]
fn on_system_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
		assert_ok!(DEBTTreasuryModule::on_system_debit(1000));
		assert_eq!(DEBTTreasuryModule::debit_pool(), 1000);
		assert_noop!(
			DEBTTreasuryModule::on_system_debit(Balance::max_value()),
			Error::<Runtime>::DebitPoolOverflow,
		);
	});
}

#[test]
fn on_system_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_ok!(DEBTTreasuryModule::on_system_surplus(1000));
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 1000);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 1000);
	});
}

#[test]
fn offset_surplus_and_debit_on_finalize_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
		assert_ok!(DEBTTreasuryModule::on_system_surplus(1000));
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 1000);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 1000);
		DEBTTreasuryModule::on_finalize(1);
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 1000);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 1000);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
		assert_ok!(DEBTTreasuryModule::on_system_debit(300));
		assert_eq!(DEBTTreasuryModule::debit_pool(), 300);
		DEBTTreasuryModule::on_finalize(2);
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 700);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 700);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
		assert_ok!(DEBTTreasuryModule::on_system_debit(800));
		assert_eq!(DEBTTreasuryModule::debit_pool(), 800);
		DEBTTreasuryModule::on_finalize(3);
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 100);
	});
}

#[test]
fn issue_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 1000);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);

		assert_ok!(DEBTTreasuryModule::issue_debit(&ALICE, 1000, true));
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 2000);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);

		assert_ok!(DEBTTreasuryModule::issue_debit(&ALICE, 1000, false));
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 3000);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 1000);
	});
}

#[test]
fn burn_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 1000);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
		assert_ok!(DEBTTreasuryModule::burn_debit(&ALICE, 300));
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 700);
		assert_eq!(DEBTTreasuryModule::debit_pool(), 0);
	});
}

#[test]
fn deposit_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_ok!(DEBTTreasuryModule::deposit_surplus(&ALICE, 300));
		assert_eq!(Currencies::free_balance(XUSD, &ALICE), 700);
		assert_eq!(Currencies::free_balance(XUSD, &DEBTTreasuryModule::account_id()), 300);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 300);
	});
}

#[test]
fn deposit_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 0);
		assert_eq!(Currencies::free_balance(BTC, &DEBTTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(
			DEBTTreasuryModule::deposit_collateral(&ALICE, BTC, 10000).is_ok(),
			false
		);
		assert_ok!(DEBTTreasuryModule::deposit_collateral(&ALICE, BTC, 500));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 500);
		assert_eq!(Currencies::free_balance(BTC, &DEBTTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 500);
	});
}

#[test]
fn withdraw_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTTreasuryModule::deposit_collateral(&ALICE, BTC, 500));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 500);
		assert_eq!(Currencies::free_balance(BTC, &DEBTTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(BTC, &BOB), 1000);
		assert_eq!(DEBTTreasuryModule::withdraw_collateral(&BOB, BTC, 501).is_ok(), false);
		assert_ok!(DEBTTreasuryModule::withdraw_collateral(&BOB, BTC, 400));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 100);
		assert_eq!(Currencies::free_balance(BTC, &DEBTTreasuryModule::account_id()), 100);
		assert_eq!(Currencies::free_balance(BTC, &BOB), 1400);
	});
}

#[test]
fn get_total_collaterals_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEBTTreasuryModule::deposit_collateral(&ALICE, BTC, 500));
		assert_eq!(DEBTTreasuryModule::get_total_collaterals(BTC), 500);
	});
}

#[test]
fn get_debit_proportion_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			DEBTTreasuryModule::get_debit_proportion(100),
			Ratio::saturating_from_rational(100, Currencies::total_issuance(XUSD))
		);
	});
}

#[test]
fn swap_collateral_not_in_auction_with_exact_stable_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EXCHANGEModule::add_liquidity(
			Origin::signed(ALICE),
			BTC,
			XUSD,
			100,
			1000
		));
		assert_eq!(DEBTTreasuryModule::total_collaterals_not_in_auction(BTC), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_ok!(DEBTTreasuryModule::deposit_collateral(&BOB, BTC, 100));
		assert_eq!(DEBTTreasuryModule::total_collaterals_not_in_auction(BTC), 100);
		assert_noop!(
			DEBTTreasuryModule::swap_collateral_not_in_auction_with_exact_stable(BTC, 499, 101, None),
			Error::<Runtime>::CollateralNotEnough,
		);

		assert_ok!(DEBTTreasuryModule::swap_collateral_not_in_auction_with_exact_stable(
			BTC, 499, 100, None
		));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 499);
	});
}

#[test]
fn swap_exact_collateral_in_auction_to_stable_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EXCHANGEModule::add_liquidity(
			Origin::signed(ALICE),
			BTC,
			XUSD,
			100,
			1000
		));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 0);
		assert_ok!(DEBTTreasuryModule::deposit_collateral(&BOB, BTC, 100));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 100);
		assert_noop!(
			DEBTTreasuryModule::swap_exact_collateral_in_auction_to_stable(BTC, 100, 500, None),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_ok!(DEBTTreasuryModule::create_collateral_auctions(
			BTC, 100, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 100);

		assert_ok!(DEBTTreasuryModule::swap_exact_collateral_in_auction_to_stable(
			BTC, 100, 500, None
		));
		assert_eq!(DEBTTreasuryModule::total_collaterals(BTC), 0);
		assert_eq!(DEBTTreasuryModule::surplus_pool(), 500);
	});
}

#[test]
fn create_collateral_auctions_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(BTC, &DEBTTreasuryModule::account_id(), 10000));
		assert_eq!(DEBTTreasuryModule::collateral_auction_maximum_size(BTC), 0);
		assert_noop!(
			DEBTTreasuryModule::create_collateral_auctions(BTC, 10001, 1000, ALICE, true),
			Error::<Runtime>::CollateralNotEnough,
		);

		// without collateral auction maximum size
		assert_ok!(DEBTTreasuryModule::create_collateral_auctions(
			BTC, 1000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 1);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1000);

		// set collateral auction maximum size
		assert_ok!(DEBTTreasuryModule::set_collateral_auction_maximum_size(
			Origin::signed(1),
			BTC,
			300
		));

		// amount < collateral auction maximum size
		// auction + 1
		assert_ok!(DEBTTreasuryModule::create_collateral_auctions(
			BTC, 200, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 2);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1200);

		// not exceed lots count cap
		// auction + 4
		assert_ok!(DEBTTreasuryModule::create_collateral_auctions(
			BTC, 1000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 6);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 2200);

		// exceed lots count cap
		// auction + 5
		assert_ok!(DEBTTreasuryModule::create_collateral_auctions(
			BTC, 2000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 11);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 4200);
	});
}

#[test]
fn auction_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(DEBTTreasuryModule::auction_surplus(Origin::signed(5), 100), BadOrigin,);
		assert_noop!(
			DEBTTreasuryModule::auction_surplus(Origin::signed(1), 100),
			Error::<Runtime>::SurplusPoolNotEnough,
		);
		assert_ok!(DEBTTreasuryModule::on_system_surplus(100));
		assert_eq!(TOTAL_SURPLUS_AUCTION.with(|v| *v.borrow_mut()), 0);
		assert_ok!(DEBTTreasuryModule::auction_surplus(Origin::signed(1), 100));
		assert_eq!(TOTAL_SURPLUS_AUCTION.with(|v| *v.borrow_mut()), 1);
	});
}

#[test]
fn auction_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			DEBTTreasuryModule::auction_debit(Origin::signed(5), 100, 200),
			BadOrigin,
		);
		assert_noop!(
			DEBTTreasuryModule::auction_debit(Origin::signed(1), 100, 200),
			Error::<Runtime>::DebitPoolNotEnough,
		);
		assert_ok!(DEBTTreasuryModule::on_system_debit(100));
		assert_eq!(TOTAL_DEBIT_AUCTION.with(|v| *v.borrow_mut()), 0);
		assert_ok!(DEBTTreasuryModule::auction_debit(Origin::signed(1), 100, 200));
		assert_eq!(TOTAL_DEBIT_AUCTION.with(|v| *v.borrow_mut()), 1);
	});
}

#[test]
fn set_collateral_auction_maximum_size_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(DEBTTreasuryModule::collateral_auction_maximum_size(BTC), 0);
		assert_noop!(
			DEBTTreasuryModule::set_collateral_auction_maximum_size(Origin::signed(5), BTC, 200),
			BadOrigin
		);
		assert_ok!(DEBTTreasuryModule::set_collateral_auction_maximum_size(
			Origin::signed(1),
			BTC,
			200
		));

		let update_collateral_auction_maximum_size_event =
			TestEvent::debt_treasury(Event::CollateralAuctionMaximumSizeUpdated(BTC, 200));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_collateral_auction_maximum_size_event));
	});
}