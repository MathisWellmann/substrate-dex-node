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

		// Check the market_info
		assert_eq!(
			crate::LiquidityPool::<Test>::get(market).unwrap(),
			MarketInfo {
				base_balance: 90_917,
				quote_balance: 109_990,
				collected_base_fees: 0,
				collected_quote_fees: 10,
			}
		);

		// Check balance of ALICE
		assert_eq!(crate::Pallet::<Test>::balance(USD, &ALICE), 890_000);
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &ALICE), 909_083);

		// Check balance of pool_account
		let pool_account = crate::Pallet::<Test>::pool_account();
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &pool_account), 90_917);
		assert_eq!(crate::Pallet::<Test>::balance(USD, &pool_account), 109_990);

		// Check balance of pool_fee_account
		let pool_fee_account = crate::Pallet::<Test>::pool_fee_account();
		assert_eq!(crate::Pallet::<Test>::balance(BTC, &pool_fee_account), 0);
		assert_eq!(crate::Pallet::<Test>::balance(USD, &pool_fee_account), 10);
	})
}
