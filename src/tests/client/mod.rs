#[cfg(test)]
mod tests {
	use ibc::core::ics02_client::{
		handler::{dispatch, ClientResult},
		msgs::{create_client::MsgCreateClient, ClientMsg},
	};
	use crate::{
		mock::{new_test_ext, Test, System},
		Context,
	};
	use ibc::{
		core::ics24_host::identifier::ClientId,
		handler::HandlerOutput,
		mock::{
			client_state::{client_type as mock_client_type, MockClientState},
			consensus_state::MockConsensusState,
			header::MockHeader,
		},
		test_utils::get_dummy_account_id,
		Height,
	};

	#[test]
	fn test_create_client_ok() {
		new_test_ext().execute_with(|| {
			let ctx = Context::<Test>::new();
            System::set_block_number(20);
			let signer = get_dummy_account_id();
			let height = Height::new(0, 42).unwrap();

			let msg = MsgCreateClient::new(
				MockClientState::new(MockHeader::new(height)).into(),
				MockConsensusState::new(MockHeader::new(height)).into(),
				signer,
			)
			.unwrap();

			let output = dispatch(&ctx, ClientMsg::CreateClient(msg.clone()));

			match output {
				Ok(HandlerOutput { result, .. }) => {
					let expected_client_id = ClientId::new(mock_client_type(), 0).unwrap();
					match result {
						ClientResult::Create(create_result) => {
							assert_eq!(create_result.client_type, mock_client_type());
							assert_eq!(create_result.client_id, expected_client_id);
							assert_eq!(
								create_result.client_state.as_ref().clone_into(),
								msg.client_state
							);
							assert_eq!(
								create_result.consensus_state.as_ref().clone_into(),
								msg.consensus_state
							);
						},
						_ => {
							panic!("unexpected result type: expected ClientResult::CreateResult!");
						},
					}
				},
				Err(err) => {
					panic!("unexpected error: {}", err);
				},
			}
		})
	}
}
