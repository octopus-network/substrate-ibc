use ibc::core::ics02_client::client_state::AnyClientState;
use ibc::core::ics02_client::context::ClientReader;
use ibc::core::ics02_client::header::AnyHeader;
use crate::context::Context;
use ibc::relayer::ics18_relayer::context::Ics18Context;
use ibc::core::ics24_host::identifier::{ClientId};
use ibc::Height;
use crate::utils::host_height;
use crate::Config;
use ibc_proto::google::protobuf::Any;
use ibc::events::IbcEvent;
use ibc::core::ics26_routing::handler::{deliver, MsgReceipt};
use ibc::signer::Signer;
use alloc::vec::Vec;
use alloc::vec;
use ibc::relayer::ics18_relayer::error::Error as ICS18Error;

impl<T: Config> Ics18Context for Context<T> {
    fn query_latest_height(&self) -> Height {
        let revision_number = 0; // TODO, in the future to fix.
        let revision_height = host_height::<T>();
        Height::new(revision_number, revision_height)
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