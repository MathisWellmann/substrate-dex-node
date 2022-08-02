use frame_support::{assert_noop, assert_ok};

use crate::{tests::*, types::MarketInfo};

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
		assert_ok!(crate::Pallet::<Test>::create_market_pool(
			origin.clone(),
			BTC,
			USD,
			100_000,
			100_000
		));

		let market = (BTC, USD);
		assert_ok!(crate::Pallet::<Test>::sell(origin, market, 10_000));

		assert_eq!(
			crate::LiquidityPool::<Test>::get(market).unwrap(),
			MarketInfo {
				base_balance: 109_990,
				quote_balance: 90_917,
				collected_base_fees: 10,
				collected_quote_fees: 0,
			}
		);

		// Check storage changes. Notice that the liquidity that ALICE has locked is also not here anymore
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &ALICE), 890_000);
		assert_eq!(crate::Pallet::<Test>::balance(USD, &ALICE), 909_083);

		// Check pool_account balances
		let pool_account = crate::Pallet::<Test>::pool_account();
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &pool_account), 109_990);
		assert_eq!(crate::Pallet::<Test>::balance(USD, &pool_account), 90_917);

		// Check pool_fee_account balances
		let pool_fee_account = crate::Pallet::<Test>::pool_fee_account();
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &pool_fee_account), 10);
		assert_eq!(crate::Pallet::<Test>::balance(USD, &pool_fee_account), 0);
	})
}
