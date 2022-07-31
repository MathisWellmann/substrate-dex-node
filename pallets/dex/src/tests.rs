use frame_support::assert_ok;

use crate::mock::*;

#[test]
fn create_market_pool_failing() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(EMPTY_ACCOUNT);
		let ret = crate::Pallet::<Test>::create_market_pool(origin, 0, 1, 100, 100);
		assert!(ret.is_err());
	})
}

#[test]
fn create_market_pool() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(ALICE);

		// Create two assets
		println!("assets: {}", <pallet_assets::Pallet<Test>>::balance(BTC, ALICE));
		assert_ok!(crate::Pallet::<Test>::create_market_pool(origin, 0, 1, 1_000_000, 1_000_000));
	})
}
