use frame_support::{assert_noop, assert_ok};

use crate::{tests::*, Error};

#[test]
fn withdraw_liquidity_no_market() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);

		let market = (BTC, USD);
		assert_noop!(
			crate::Pallet::<Test>::withdraw_liquidity(origin, market, 100, 100),
			Error::<Test>::MarketDoesNotExist
		);
	})
}

#[test]
fn withdraw_liquidity_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let origin_alice = Origin::signed(ALICE);
		let base_asset = BTC;
		let quote_asset = USD;
		let market = (base_asset, quote_asset);

		assert_ok!(crate::Pallet::<Test>::create_market_pool(
			origin_alice,
			base_asset,
			quote_asset,
			100,
			100
		));

		let origin_bob = Origin::signed(BOB);
		// This will obviously not work as BOB has not yet deposited anything into the pool
		assert_noop!(
			crate::Pallet::<Test>::withdraw_liquidity(origin_bob, market, 100, 100),
			Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]
fn withdraw_liquidity() {
	new_test_ext().execute_with(|| {
		let origin_alice = Origin::signed(ALICE);
		let base_asset = BTC;
		let quote_asset = USD;
		let market = (base_asset, quote_asset);

		assert_ok!(crate::Pallet::<Test>::create_market_pool(
			origin_alice.clone(),
			base_asset,
			quote_asset,
			100_000,
			100_000
		));

		assert_ok!(crate::Pallet::<Test>::withdraw_liquidity(origin_alice, market, 50_000, 50_000));

		// check balances
		assert_eq!(crate::Pallet::<Test>::balance(base_asset, &ALICE), 950_000);
		assert_eq!(crate::Pallet::<Test>::balance(quote_asset, &ALICE), 950_000);

		// check LiqProvisionPool changes
		assert_eq!(crate::LiqProvisionPool::<Test>::get(market, ALICE), (50_000, 50_000));
	})
}

// TODO: there should be a test to ensure that withdrawing liquidity does not destroy the pool and set the balances to zero
