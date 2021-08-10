use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

//
// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }
//
// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			TemplateModule::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }

#[test]
fn test_hashing() {
	use sha2::Digest;

	let hello = "hello world";

	let r = sha2::Sha256::digest(hello.as_bytes());

	let l = sp_core::hashing::sha2_256(hello.as_bytes());
	assert_eq!(format!("{:?}", r), format!("{:?}", l));

	let mut tmp = String::new();
	for item in l.iter() {
		tmp.push_str(&format!("{:02x}", item));
	}
	assert_eq!(format!("{:x}", r), tmp);
}
