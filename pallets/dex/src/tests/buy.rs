use frame_support::assert_ok;

use super::*;

#[test]
fn buy_no_pool() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let market = (BTC, XMR);
		let ret = crate::Pallet::<Test>::buy(origin, market, 100);
		// This should error as there is no liquidity pool created yet
		assert!(ret.is_err());
	})
}

#[test]
fn buy_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin.clone(), BTC, XMR, 100, 100));

		let market = (BTC, XMR);
		// This should obviously fail as ALICE does not have enough balance
		let ret = crate::Pallet::<Test>::buy(origin, market, u128::MAX);
		assert!(ret.is_err());
	})
}

#[test]
fn buy() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin.clone(), BTC, XMR, 100, 100));

		let market = (BTC, XMR);
		assert_ok!(crate::Pallet::<Test>::buy(origin, market, 10));
	})
}
