use crate::{tests::*, types::OrderType};

#[test]
fn get_received_amount() {
	new_test_ext().execute_with(|| {
		let base_amount = 100;
		let quote_amount = 100;

		let receive_amount = crate::Pallet::<Test>::get_received_amount(
			base_amount,
			quote_amount,
			OrderType::Buy,
			10,
		)
		.unwrap();
		println!("receive_amount: {}", receive_amount);
		assert_eq!(receive_amount, 10);

		let receive_amount = crate::Pallet::<Test>::get_received_amount(
			base_amount,
			quote_amount,
			OrderType::Buy,
			100,
		)
		.unwrap();
		println!("receive_amount: {}", receive_amount);
		assert_eq!(receive_amount, 50);

		let receive_amount = crate::Pallet::<Test>::get_received_amount(
			base_amount,
			quote_amount,
			OrderType::Sell,
			10,
		)
		.unwrap();
		println!("receive_amount: {}", receive_amount);
		assert_eq!(receive_amount, 10);

		let receive_amount = crate::Pallet::<Test>::get_received_amount(
			base_amount,
			quote_amount,
			OrderType::Sell,
			100,
		)
		.unwrap();
		println!("receive_amount: {}", receive_amount);
		assert_eq!(receive_amount, 50);
	})
}
