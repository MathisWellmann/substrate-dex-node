use frame_support::assert_ok;

use super::*;

#[test]
fn create_market_pool_failing() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(EMPTY_ACCOUNT);
		let ret = crate::Pallet::<Test>::create_market_pool(origin, BTC, XMR, 100, 100);
		assert!(ret.is_err());
	})
}

#[test]
fn create_market_pool() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);

		// Create two assets
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin, BTC, XMR, 100, 100));

		// Check storage changes
		assert_eq!(<crate::LiquidityPool::<Test>>::get((BTC, XMR)).unwrap(), (100, 100));
	})
}
