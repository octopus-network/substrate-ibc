use super::{Module, Store, Trait};
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::storage::StorageMap;
use ibc::ics02_client::client_def::{AnyClientState, AnyConsensusState};
use ibc::ics02_client::client_type::ClientType;
use ibc::ics02_client::context::{ClientKeeper, ClientReader};
use ibc::ics02_client::error::Error as ICS02Error;
use ibc::ics03_connection::connection::ConnectionEnd;
use ibc::ics03_connection::context::{ConnectionKeeper, ConnectionReader};
use ibc::ics03_connection::error::Error as ICS03Error;
use ibc::ics23_commitment::commitment::CommitmentPrefix;
use ibc::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::ics26_routing::context::ICS26Context;
use ibc::Height;
use sp_runtime::RuntimeDebug;
use sp_std::{if_std, prelude::*};
use std::str::FromStr;
use tendermint_proto::Protobuf;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Any {
    pub type_url: String,
    pub value: Vec<u8>,
}

#[derive(Clone)]
pub struct Context<T: Trait> {
    pub _pd: PhantomData<T>,
    pub tmp: u8,
}

impl<T: Trait> ICS26Context for Context<T> {}

impl<T: Trait> ClientReader for Context<T> {
    fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {
        if_std! {
            println!("in read client_type");
        }
        Some(ClientType::GRANDPA)
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        if_std! {
            println!("in read client_state");
        }
        if <Module<T> as Store>::ClientStatesV2::contains_key(client_id.as_bytes()) {
            let data = <Module<T> as Store>::ClientStatesV2::get(client_id.as_bytes());
            Some(AnyClientState::decode_vec(&*data).unwrap())
        } else {
            if_std! {
                println!("read client_state returns None");
            }
            None
        }
    }

    fn consensus_state(&self, client_id: &ClientId, height: Height) -> Option<AnyConsensusState> {
        if_std! {
            println!("in read consensus_state");
        }
        let height = height.encode_vec().unwrap();
        if <Module<T> as Store>::ConsensusStatesV2::contains_key((client_id.as_bytes(), &height)) {
            let data = <Module<T> as Store>::ConsensusStatesV2::get((client_id.as_bytes(), height));
            Some(AnyConsensusState::decode_vec(&*data).unwrap())
        } else {
            if_std! {
                println!("read consensus_state returns None");
            }
            None
        }
    }
}

impl<T: Trait> ClientKeeper for Context<T> {
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), ICS02Error> {
        if_std! {
            println!("in store_client_type");
        }
        Ok(())
    }

    fn next_client_id(&mut self) -> ClientId {
        // TODO
        if self.tmp == 0 {
            ClientId::from_str("appia-client-id").unwrap()
        } else {
            ClientId::from_str("flaminia-client-id").unwrap()
        }
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> Result<(), ICS02Error> {
        if_std! {
            println!("in store_client_state");
        }
        let data = client_state.encode_vec().unwrap();
        <Module<T> as Store>::ClientStatesV2::insert(client_id.as_bytes(), data);
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: AnyConsensusState,
    ) -> Result<(), ICS02Error> {
        if_std! {
            println!("in store_consensus_state");
        }
        let height = height.encode_vec().unwrap();
        let data = consensus_state.encode_vec().unwrap();
        <Module<T> as Store>::ConsensusStatesV2::insert((client_id.as_bytes(), height), data);
        Ok(())
    }
}

impl<T: Trait> ConnectionReader for Context<T> {
    fn connection_end(&self, conn_id: &ConnectionId) -> Option<ConnectionEnd> {
        None
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        None
    }

    fn host_current_height(&self) -> Height {
        Height::zero()
    }

    fn host_chain_history_size(&self) -> usize {
        0
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        vec![0].into()
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Option<AnyConsensusState> {
        None
    }

    fn host_consensus_state(&self, height: Height) -> Option<AnyConsensusState> {
        None
    }
}

impl<T: Trait> ConnectionKeeper for Context<T> {
    fn next_connection_id(&mut self) -> ConnectionId {
        ConnectionId::from_str("todo").unwrap()
    }

    fn store_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection_end: &ConnectionEnd,
    ) -> Result<(), ICS03Error> {
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        connection_id: &ConnectionId,
        client_id: &ClientId,
    ) -> Result<(), ICS03Error> {
        Ok(())
    }
}
