mod buy;
mod create_pool;
mod deposit_liqudity;
mod fee_from_amount;
mod get_received_amount;
mod mock;
mod sell;
mod withdraw_liquidity;

pub use mock::*;

/// Just experimenting
#[test]
fn pallet_account() {
	new_test_ext().execute_with(|| {
		let pool_account = crate::Pallet::<Test>::pool_account();
		let bytes: &[u8; 32] = pool_account.as_ref();
		println!("pool_account: {:?}", bytes);
	})
}

#[test]
fn pallet_fee_account() {
	new_test_ext().execute_with(|| {
		let pool_sub_account = crate::Pallet::<Test>::pool_fee_account();
		println!("pool_sub_account: {:?}", pool_sub_account);
	})
}
