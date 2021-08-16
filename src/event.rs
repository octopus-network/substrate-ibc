use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;


pub mod primitive {
    use sp_std::str::FromStr;

    use ibc::ics02_client::client_type::ClientType as IbcClientType;
    use ibc::ics02_client::height::Height as IbcHeight;
    use ibc::ics24_host::identifier::ClientId as IbcClientId;
    use ibc::ics24_host::identifier::ConnectionId as IbcConnectionId;
    use ibc::ics24_host::identifier::PortId as IbcPortId;
    use ibc::ics24_host::identifier::ChannelId as IbcChannelId;
    use ibc::ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence};
    use ibc::timestamp::Timestamp as IbcTimestamp;


    use codec::{Decode, Encode};
    use sp_runtime::RuntimeDebug;

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct PortId(pub String);

    impl PortId {
        fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcPortId> for PortId {
        fn from(value : IbcPortId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl PortId {
        pub fn to_ibc_port_id(self) -> IbcPortId {
            IbcPortId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ChannelId(pub String);

    impl ChannelId {
        fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl From<IbcChannelId> for ChannelId {
        fn from(value : IbcChannelId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl ChannelId {
        pub fn to_ibc_channel_id(self) -> IbcChannelId {
            IbcChannelId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct Height {
        /// Previously known as "epoch"
        pub revision_number: u64,

        /// The height of a block
        pub revision_height: u64,
    }

    impl From<IbcHeight> for Height {
        fn from(
            IbcHeight {
                revision_number,
                revision_height,
            }: IbcHeight,
        ) -> Self {
            Height {
                revision_number,
                revision_height,
            }
        }
    }

    impl Height {
        pub fn to_ibc_height(self) -> IbcHeight {
            IbcHeight {
                revision_number: self.revision_number,
                revision_height: self.revision_height,
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub enum ClientType {
        Tendermint,
        Grandpa,
    }

    impl From<IbcClientType> for ClientType {
        fn from(value: IbcClientType) -> Self {
            match value {
                IbcClientType::Tendermint => ClientType::Tendermint,
                IbcClientType::Grandpa => ClientType::Grandpa,

                _ => unreachable!(),
            }
        }
    }


    impl ClientType {
        pub fn to_ibc_client_type(self) -> IbcClientType {
            match self {
                ClientType::Tendermint => IbcClientType::Tendermint,
                ClientType::Grandpa => IbcClientType::Grandpa,
                _ => unreachable!(),
            }
        }
    }


    impl ClientType {
        pub fn to_ibc_client_type(self) -> IbcClientType {
            match self {
                ClientType::Tendermint => IbcClientType::Tendermint,
                ClientType::Grandpa => IbcClientType::Grandpa,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ClientId(pub String);

    impl ClientId {
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcClientId> for ClientId {
        fn from(value: IbcClientId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl ClientId {
        pub fn to_ibc_client_id(self) -> IbcClientId {
            IbcClientId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ConnectionId(String);

    impl ConnectionId {
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcConnectionId> for ConnectionId {
        fn from(value: IbcConnectionId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl ConnectionId {
        pub fn to_ibc_connection_id(self) -> IbcConnectionId {
            IbcConnectionId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct Timestamp {
        time: String,
    }

    impl From<IbcTimestamp> for Timestamp {
        fn from(val : IbcTimestamp) -> Self {
            Self {
                time: val.to_string(),
            }
        }
    }

    impl Timestamp {
        pub fn to_ibc_timestamp(self) -> IbcTimestamp {
            let timestamp = IbcTimestamp::from_str(self.time.as_str()).unwrap();
            timestamp
        }

    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct Sequence(u64);

    impl From<IbcSequence> for Sequence {
        fn from(val : IbcSequence) -> Self {
            Self(val.0)
        }
    }


    impl Sequence {
        pub fn to_ibc_sequence(self) -> IbcSequence {
            IbcSequence(self.0)
        }
    }


    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct Packet {
        pub sequence: Sequence,
        pub source_port: PortId,
        pub source_channel: ChannelId,
        pub destination_port: PortId,
        pub destination_channel: ChannelId,
        pub data: Vec<u8>,
        pub timeout_height: Height,
        pub timeout_timestamp: Timestamp,
    }

    impl From<IbcPacket> for Packet {
        fn from(val: IbcPacket) -> Self {
            Self {
                sequence: val.sequence.into(),
                source_port: val.source_port.into(),
                source_channel: val.source_channel.into(),
                destination_port: val.destination_port.into(),
                destination_channel: val.destination_channel.into(),
                data: val.data,
                timeout_height: val.timeout_height.into(),
                timeout_timestamp: val.timeout_timestamp.into(),
            }
        }
    }


    impl Packet {
        pub fn to_ibc_packet(self) -> IbcPacket {
            IbcPacket {
                sequence: self.sequence.to_ibc_sequence(),
                source_port: self.source_port.to_ibc_port_id(),
                source_channel: self.source_channel.to_ibc_channel_id(),
                destination_port: self.destination_port.to_ibc_port_id(),
                destination_channel: self.destination_channel.to_ibc_channel_id(),
                data: self.data,
                timeout_height: self.timeout_height.to_ibc_height(),
                timeout_timestamp: IbcTimestamp::now(),
                // timeout_timestamp: self.timeout_timestamp.to_ibc_timestamp(),
            }
        }
    }



}
