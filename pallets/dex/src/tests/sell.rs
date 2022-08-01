use frame_support::{assert_noop, assert_ok};

use crate::tests::*;

#[test]
fn sell_no_pool() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let market = (BTC, USD);
		assert_noop!(
			crate::Pallet::<Test>::sell(origin, market, 100),
			crate::Error::<Test>::MarketDoesNotExist
		);
	})
}

#[test]
fn sell_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin.clone(), BTC, XMR, 100, 100));

		let market = (BTC, XMR);
		assert_noop!(
			crate::Pallet::<Test>::sell(origin, market, u128::MAX),
			crate::Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]
fn sell() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin.clone(), BTC, USD, 100, 100));

		let market = (BTC, USD);
		assert_ok!(crate::Pallet::<Test>::sell(origin, market, 10));

		// Check storage changes. Notice that the liquidity that ALICE has locked is also not here anymore
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &ALICE), 890);
		assert_eq!(crate::Pallet::<Test>::balance(USD, &ALICE), 910);
	})
}
