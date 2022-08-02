use frame_support::assert_ok;

use crate::types::MarketInfo;

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
		let base_asset = BTC;
		let quote_asset = USD;
		let market = (base_asset, quote_asset);

		// Create two assets
		assert_ok!(crate::Pallet::<Test>::create_market_pool(
			origin,
			base_asset,
			quote_asset,
			100,
			100
		));

		// Check LiquidityPool storage changes
		assert_eq!(
			<crate::LiquidityPool::<Test>>::get(market).unwrap(),
			MarketInfo {
				base_balance: 100,
				quote_balance: 100,
				collected_base_fees: 0,
				collected_quote_fees: 0,
			}
		);

		// Check LiqProvisionPool storage changes
		assert_eq!(crate::LiqProvisionPool::<Test>::get(market, ALICE), (100, 100));
	})
}
