//! Unit tests for the incentives module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use orml_rewards::PoolInfo;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

#[test]
fn deposit_exchange_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(TokensModule::deposit(BTC_AUSD_LP, &ALICE, 10000));
		assert_eq!(TokensModule::free_balance(BTC_AUSD_LP, &ALICE), 10000);
		assert_eq!(
			TokensModule::free_balance(BTC_AUSD_LP, &IncentivesModule::account_id()),
			0
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC_AUSD_LP), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC_AUSD_LP), ALICE),
			(0, 0)
		);

		assert_ok!(IncentivesModule::deposit_exchange_share(
			Origin::signed(ALICE),
			BTC_AUSD_LP,
			10000
		));
		let deposit_exchange_share_event =
			TestEvent::incentives(RawEvent::DepositEXCHANGEShare(ALICE, BTC_AUSD_LP, 10000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == deposit_exchange_share_event));

		assert_eq!(TokensModule::free_balance(BTC_AUSD_LP, &ALICE), 0);
		assert_eq!(
			TokensModule::free_balance(BTC_AUSD_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC_AUSD_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC_AUSD_LP), ALICE),
			(10000, 0)
		);
	});
}

#[test]
fn withdraw_exchange_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(TokensModule::deposit(BTC_AUSD_LP, &ALICE, 10000));

		assert_noop!(
			IncentivesModule::withdraw_exchange_share(Origin::signed(BOB), BTC_AUSD_LP, 10000),
			Error::<Runtime>::NotEnough,
		);

		assert_ok!(IncentivesModule::deposit_exchange_share(
			Origin::signed(ALICE),
			BTC_AUSD_LP,
			10000
		));
		assert_eq!(TokensModule::free_balance(BTC_AUSD_LP, &ALICE), 0);
		assert_eq!(
			TokensModule::free_balance(BTC_AUSD_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC_AUSD_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC_AUSD_LP), ALICE),
			(10000, 0)
		);

		assert_ok!(IncentivesModule::withdraw_exchange_share(
			Origin::signed(ALICE),
			BTC_AUSD_LP,
			8000
		));
		let withdraw_exchange_share_event =
			TestEvent::incentives(RawEvent::WithdrawEXCHANGEShare(ALICE, BTC_AUSD_LP, 8000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == withdraw_exchange_share_event));

		assert_eq!(TokensModule::free_balance(BTC_AUSD_LP, &ALICE), 8000);
		assert_eq!(
			TokensModule::free_balance(BTC_AUSD_LP, &IncentivesModule::account_id()),
			2000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC_AUSD_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC_AUSD_LP), ALICE),
			(2000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC_AUSD_LP), ALICE),
			(2000, 0)
		);
	});
}

#[test]
fn update_lend_incentive_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_lend_incentive_rewards(Origin::signed(ALICE), vec![]),
			BadOrigin
		);
		assert_eq!(IncentivesModule::lend_incentive_rewards(BTC), 0);
		assert_eq!(IncentivesModule::lend_incentive_rewards(DOT), 0);

		assert_ok!(IncentivesModule::update_lend_incentive_rewards(
			Origin::signed(4),
			vec![(BTC, 200), (DOT, 1000),],
		));
		assert_eq!(IncentivesModule::lend_incentive_rewards(BTC), 200);
		assert_eq!(IncentivesModule::lend_incentive_rewards(DOT), 1000);

		assert_ok!(IncentivesModule::update_lend_incentive_rewards(
			Origin::signed(4),
			vec![(BTC, 100), (BTC, 300), (BTC, 500),],
		));
		assert_eq!(IncentivesModule::lend_incentive_rewards(BTC), 500);
	});
}

#[test]
fn update_exchange_incentive_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_exchange_incentive_rewards(Origin::signed(ALICE), vec![]),
			BadOrigin
		);
		assert_noop!(
			IncentivesModule::update_exchange_incentive_rewards(Origin::signed(4), vec![(BTC, 200), (DOT, 1000)],),
			Error::<Runtime>::InvalidCurrencyId
		);

		assert_eq!(IncentivesModule::exchange_incentive_rewards(BTC_AUSD_LP), 0);
		assert_eq!(IncentivesModule::exchange_incentive_rewards(DOT_AUSD_LP), 0);

		assert_ok!(IncentivesModule::update_exchange_incentive_rewards(
			Origin::signed(4),
			vec![(BTC_AUSD_LP, 200), (DOT_AUSD_LP, 1000)],
		));
		assert_eq!(IncentivesModule::exchange_incentive_rewards(BTC_AUSD_LP), 200);
		assert_eq!(IncentivesModule::exchange_incentive_rewards(DOT_AUSD_LP), 1000);

		assert_ok!(IncentivesModule::update_exchange_incentive_rewards(
			Origin::signed(4),
			vec![(BTC_AUSD_LP, 100), (BTC_AUSD_LP, 300), (BTC_AUSD_LP, 500),],
		));
		assert_eq!(IncentivesModule::exchange_incentive_rewards(BTC_AUSD_LP), 500);
	});
}

#[test]
fn update_stake_earning_incentive_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_stake_earning_incentive_reward(Origin::signed(ALICE), 100),
			BadOrigin
		);
		assert_eq!(IncentivesModule::stake_earning_incentive_reward(), 0);

		assert_ok!(IncentivesModule::update_stake_earning_incentive_reward(Origin::signed(4), 100));
		assert_eq!(IncentivesModule::stake_earning_incentive_reward(), 100);
	});
}

#[test]
fn update_exchange_saving_rates_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_exchange_saving_rates(Origin::signed(ALICE), vec![]),
			BadOrigin
		);

		assert_noop!(
			IncentivesModule::update_exchange_saving_rates(
				Origin::signed(4),
				vec![(BTC, Rate::saturating_from_rational(1, 10000)),],
			),
			Error::<Runtime>::InvalidCurrencyId
		);

		assert_eq!(IncentivesModule::exchange_saving_rates(BTC_AUSD_LP), Rate::zero());
		assert_eq!(IncentivesModule::exchange_saving_rates(DOT_AUSD_LP), Rate::zero());

		assert_ok!(IncentivesModule::update_exchange_saving_rates(
			Origin::signed(4),
			vec![
				(BTC_AUSD_LP, Rate::saturating_from_rational(1, 10000)),
				(DOT_AUSD_LP, Rate::saturating_from_rational(1, 5000)),
			],
		));
		assert_eq!(
			IncentivesModule::exchange_saving_rates(BTC_AUSD_LP),
			Rate::saturating_from_rational(1, 10000)
		);
		assert_eq!(
			IncentivesModule::exchange_saving_rates(DOT_AUSD_LP),
			Rate::saturating_from_rational(1, 5000)
		);

		assert_ok!(IncentivesModule::update_exchange_saving_rates(
			Origin::signed(4),
			vec![
				(BTC_AUSD_LP, Rate::saturating_from_rational(1, 20000)),
				(BTC_AUSD_LP, Rate::saturating_from_rational(1, 30000)),
				(BTC_AUSD_LP, Rate::saturating_from_rational(1, 40000)),
			],
		));
		assert_eq!(
			IncentivesModule::exchange_saving_rates(BTC_AUSD_LP),
			Rate::saturating_from_rational(1, 40000)
		);
	});
}

#[test]
fn on_add_liquidity_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), ALICE),
			(0, 0)
		);

		OnAddLiquidity::<Runtime>::happened(&(ALICE, BTC, 100));
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC)),
			PoolInfo {
				total_shares: 100,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC)),
			PoolInfo {
				total_shares: 100,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), ALICE),
			(100, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), ALICE),
			(100, 0)
		);

		OnAddLiquidity::<Runtime>::happened(&(BOB, BTC, 100));
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC)),
			PoolInfo {
				total_shares: 200,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC)),
			PoolInfo {
				total_shares: 200,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), BOB),
			(100, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), BOB),
			(100, 0)
		);
	});
}

#[test]
fn on_remove_liquidity_works() {
	ExtBuilder::default().build().execute_with(|| {
		OnAddLiquidity::<Runtime>::happened(&(ALICE, BTC, 100));
		OnAddLiquidity::<Runtime>::happened(&(BOB, BTC, 100));
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC)),
			PoolInfo {
				total_shares: 200,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC)),
			PoolInfo {
				total_shares: 200,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), ALICE),
			(100, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), ALICE),
			(100, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), BOB),
			(100, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), BOB),
			(100, 0)
		);

		OnRemoveLiquidity::<Runtime>::happened(&(ALICE, BTC, 40));
		OnRemoveLiquidity::<Runtime>::happened(&(BOB, BTC, 70));
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeIncentive(BTC)),
			PoolInfo {
				total_shares: 90,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::ExchangeSaving(BTC)),
			PoolInfo {
				total_shares: 90,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), ALICE),
			(60, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), ALICE),
			(60, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeIncentive(BTC), BOB),
			(30, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::ExchangeSaving(BTC), BOB),
			(30, 0)
		);
	});
}

#[test]
fn on_update_loan_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			RewardsModule::pools(PoolId::Lend(BTC)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::Lend(BTC), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::Lend(BTC), BOB),
			(0, 0)
		);

		OnUpdateLoan::<Runtime>::happened(&(ALICE, BTC, 100, 0));
		assert_eq!(
			RewardsModule::pools(PoolId::Lend(BTC)),
			PoolInfo {
				total_shares: 100,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::Lend(BTC), ALICE),
			(100, 0)
		);

		OnUpdateLoan::<Runtime>::happened(&(BOB, BTC, 100, 500));
		assert_eq!(
			RewardsModule::pools(PoolId::Lend(BTC)),
			PoolInfo {
				total_shares: 700,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::Lend(BTC), BOB),
			(600, 0)
		);

		OnUpdateLoan::<Runtime>::happened(&(ALICE, BTC, -50, 100));
		assert_eq!(
			RewardsModule::pools(PoolId::Lend(BTC)),
			PoolInfo {
				total_shares: 650,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::Lend(BTC), ALICE),
			(50, 0)
		);

		OnUpdateLoan::<Runtime>::happened(&(BOB, BTC, -650, 600));
		assert_eq!(
			RewardsModule::pools(PoolId::Lend(BTC)),
			PoolInfo {
				total_shares: 50,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::Lend(BTC), BOB),
			(0, 0)
		);
	});
}

#[test]
fn pay_out_works_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(DOS, &LendIncentivePool::get(), 10000));
		assert_ok!(TokensModule::deposit(DOS, &ExchangeIncentivePool::get(), 10000));
		assert_ok!(TokensModule::deposit(AUSD, &ExchangeIncentivePool::get(), 10000));
		assert_ok!(TokensModule::deposit(DOS, &Stake_EarningIncentivePool::get(), 10000));

		assert_eq!(TokensModule::free_balance(DOS, &LendIncentivePool::get()), 10000);
		assert_eq!(TokensModule::free_balance(DOS, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::Lend(BTC), 1000);
		assert_eq!(TokensModule::free_balance(DOS, &LendIncentivePool::get()), 9000);
		assert_eq!(TokensModule::free_balance(DOS, &ALICE), 1000);

		assert_eq!(TokensModule::free_balance(DOS, &ExchangeIncentivePool::get()), 10000);
		assert_eq!(TokensModule::free_balance(DOS, &BOB), 0);
		IncentivesModule::payout(&BOB, PoolId::ExchangeIncentive(BTC), 1000);
		assert_eq!(TokensModule::free_balance(DOS, &ExchangeIncentivePool::get()), 9000);
		assert_eq!(TokensModule::free_balance(DOS, &BOB), 1000);

		assert_eq!(TokensModule::free_balance(AUSD, &ExchangeIncentivePool::get()), 10000);
		assert_eq!(TokensModule::free_balance(AUSD, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::ExchangeSaving(BTC), 1000);
		assert_eq!(TokensModule::free_balance(AUSD, &ExchangeIncentivePool::get()), 9000);
		assert_eq!(TokensModule::free_balance(AUSD, &ALICE), 1000);

		assert_eq!(TokensModule::free_balance(DOS, &Stake_EarningIncentivePool::get()), 10000);
		assert_eq!(TokensModule::free_balance(DOS, &BOB), 1000);
		IncentivesModule::payout(&BOB, PoolId::Stake_Earning, 3000);
		assert_eq!(TokensModule::free_balance(DOS, &Stake_EarningIncentivePool::get()), 7000);
		assert_eq!(TokensModule::free_balance(DOS, &BOB), 4000);
	});
}

#[test]
fn accumulate_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(IncentivesModule::update_lend_incentive_rewards(
			Origin::signed(4),
			vec![(BTC, 1000), (DOT, 2000),],
		));
		assert_ok!(IncentivesModule::update_exchange_incentive_rewards(
			Origin::signed(4),
			vec![(BTC_AUSD_LP, 100), (DOT_AUSD_LP, 200),],
		));
		assert_ok!(IncentivesModule::update_stake_earning_incentive_reward(Origin::signed(4), 30));
		assert_ok!(IncentivesModule::update_exchange_saving_rates(
			Origin::signed(4),
			vec![
				(BTC_AUSD_LP, Rate::saturating_from_rational(1, 100)),
				(DOT_AUSD_LP, Rate::saturating_from_rational(1, 100)),
			],
		));

		assert_eq!(IncentivesModule::accumulate_reward(10, |_, _| {}), vec![]);

		RewardsModule::add_share(&ALICE, PoolId::Lend(BTC), 1);
		assert_eq!(IncentivesModule::accumulate_reward(20, |_, _| {}), vec![(DOS, 1000)]);

		RewardsModule::add_share(&ALICE, PoolId::Lend(DOT), 1);
		assert_eq!(IncentivesModule::accumulate_reward(30, |_, _| {}), vec![(DOS, 3000)]);

		RewardsModule::add_share(&ALICE, PoolId::ExchangeIncentive(BTC_AUSD_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::ExchangeSaving(BTC_AUSD_LP), 1);
		assert_eq!(
			IncentivesModule::accumulate_reward(40, |_, _| {}),
			vec![(DOS, 3100), (AUSD, 5)]
		);

		RewardsModule::add_share(&ALICE, PoolId::ExchangeIncentive(DOT_AUSD_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::ExchangeSaving(DOT_AUSD_LP), 1);
		assert_eq!(
			IncentivesModule::accumulate_reward(50, |_, _| {}),
			vec![(DOS, 3300), (AUSD, 9)]
		);

		RewardsModule::add_share(&ALICE, PoolId::Stake_Earning, 1);
		assert_eq!(
			IncentivesModule::accumulate_reward(50, |_, _| {}),
			vec![(DOS, 3330), (AUSD, 9)]
		);

		assert_eq!(IncentivesModule::accumulate_reward(59, |_, _| {}), vec![]);

		mock_shutdown();
		assert_eq!(IncentivesModule::accumulate_reward(60, |_, _| {}), vec![]);
	});
}
