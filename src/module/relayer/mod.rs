use crate::{
	alloc::string::ToString, context::Context, utils::host_height, Config, REVISION_NUMBER,
};
use ibc::{
	core::{
		ics02_client::{client_state::AnyClientState, context::ClientReader, header::AnyHeader},
		ics24_host::identifier::ClientId,
		ics26_routing::handler::{deliver, MsgReceipt},
	},
	events::IbcEvent,
	relayer::ics18_relayer::{context::Ics18Context, error::Error as ICS18Error},
	signer::Signer,
	Height,
};
use ibc_proto::google::protobuf::Any;
use scale_info::prelude::{vec, vec::Vec};

impl<T: Config> Ics18Context for Context<T> {
	fn query_latest_height(&self) -> Height {
		let revision_height = host_height::<T>();
		Height::new(REVISION_NUMBER, revision_height).expect(&REVISION_NUMBER.to_string())
	}

	fn query_client_full_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		// Forward call to Ics2.
		ClientReader::client_state(self, client_id).ok()
	}

	fn query_latest_header(&self) -> Option<AnyHeader> {
		todo!()
	}

	fn send(&mut self, msgs: Vec<Any>) -> Result<Vec<IbcEvent>, ICS18Error> {
		// Forward call to Ics26 delivery method.
		let mut all_events = vec![];
		for msg in msgs {
			let MsgReceipt { mut events, .. } =
				deliver(self, msg).map_err(ICS18Error::transaction_failed)?;
			all_events.append(&mut events);
		}
		Ok(all_events)
	}

	fn signer(&self) -> Signer {
		"0CDA3F47EF3C4906693B170EF650EB968C5F4B2C".parse().unwrap()
	}
}
