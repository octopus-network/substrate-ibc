use super::pallet::ConsensusStates;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::Any;
use sp_keyring::{sr25519::Keyring, AccountKeyring};
use super::*;

// for substrate-ibc
// 1. create single grandpa client
// 2. create multi grandpa client
// 3. update multi grandpa client
// 4. upgrade grandpa client

// for ibc-rs
// 5. client_type need tests
// 6. conn_open_init_msg_processing
// 7. conn_open_try_msg_processing
// 8. conn_open_confirm_msg_processing
// 9. conn_open_ack_msg_processing

// 10. ack_packet_processing
// 11. chan_open_ack_msg_processing
// 12. chan_open_confirm_msg_processing
// 13. chan_open_init_msg_processing
// 14. chan_open_try_msg_processing
// 15. send_packet_processing
// 16. recv_packet_processing
// 17. timeout_packet_processing
// 18. timeout_on_close_packet_processing
// 19. write_ack_packet_processing

// 20. routing_module_and_keepers




#[test]
fn test_deliver() {
	let alice: AccountId = AccountKeyring::Alice.into();
	let origin = Origin::signed(alice);
	let any: Any = Any { type_url: vec![1,2,3], value: vec![1,2,3] };
	let tmp: u8 = 0;
	new_test_ext().execute_with(|| {
		assert_ok!(Ibc::deliver(origin, vec![any], tmp));
	})
}
