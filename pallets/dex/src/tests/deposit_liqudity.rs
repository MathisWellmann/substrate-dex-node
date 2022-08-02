use frame_support::{assert_noop, assert_ok};

use crate::{tests::*, Error};

#[test]
fn deposit_liquidity_no_market() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let market = (BTC, USD);

		assert_noop!(
			crate::Pallet::<Test>::deposit_liquidity(origin, market, 100, 100),
			Error::<Test>::MarketDoesNotExist
		);
	})
}

#[test]
fn deposit_liquidity_no_enough_balance() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let base_asset = BTC;
		let quote_asset = USD;
		let market = (base_asset, quote_asset);

		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin.clone(), BTC, USD, 100, 100));

		assert_noop!(
			crate::Pallet::<Test>::deposit_liquidity(origin, market, u128::MAX, u128::MAX),
			Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]
fn deposit_liquidity() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let base_asset = BTC;
		let quote_asset = USD;
		let market = (base_asset, quote_asset);

		assert_ok!(crate::Pallet::<Test>::create_market_pool(
			origin.clone(),
			BTC,
			USD,
			100_000,
			100_000
		));
		assert_ok!(crate::Pallet::<Test>::deposit_liquidity(origin, market, 100_000, 100_000));

		// Check user balance changes
		assert_eq!(crate::Pallet::<Test>::balance(base_asset, &ALICE), 800_000);
		assert_eq!(crate::Pallet::<Test>::balance(quote_asset, &ALICE), 800_000);

		// Check LiqProvisionPool storage
		assert_eq!(crate::LiqProvisionPool::<Test>::get(market, ALICE), (200_000, 200_000));
	})
}
