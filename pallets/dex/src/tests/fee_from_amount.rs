use crate::tests::*;

#[test]
fn fee_from_amount() {
	new_test_ext().execute_with(|| {
		assert_eq!(crate::Pallet::<Test>::fee_from_amount(1_000_000).unwrap(), 1_000);
	})
}
