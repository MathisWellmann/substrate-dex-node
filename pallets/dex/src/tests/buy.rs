use frame_support::{assert_noop, assert_ok};

use crate::types::MarketInfo;

use super::*;

#[test]
fn buy_no_pool() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let market = (BTC, XMR);
		assert_noop!(
			crate::Pallet::<Test>::buy(origin, market, 100),
			crate::Error::<Test>::MarketDoesNotExist
		);
	})
}

#[test]
fn buy_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin.clone(), BTC, XMR, 100, 100));

		let market = (BTC, XMR);
		// This should obviously fail as ALICE does not have enough balance
		assert_noop!(
			crate::Pallet::<Test>::buy(origin, market, u128::MAX),
			crate::Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]
fn buy() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(crate::Pallet::<Test>::create_market_pool(
			origin.clone(),
			BTC,
			USD,
			100_000,
			100_000
		));

		let market = (BTC, USD);
		assert_ok!(crate::Pallet::<Test>::buy(origin, market, 10_000));

		// Check the storage changes
		// Notice that both the liquidity deposit and the payed amount are gone from USD balance
		assert_eq!(crate::Pallet::<Test>::balance(USD, &ALICE), 890_000);
		// Notice how 100 BTC balance also went into the liquidity pool
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &ALICE), 910_000);

		// Check the market_info
		assert_eq!(
			crate::LiquidityPool::<Test>::get(market).unwrap(),
			MarketInfo {
				base_balance: 90_000,
				quote_balance: 110_000,
				collected_base_fees: 0,
				collected_quote_fees: 1
			}
		);
	})
}
