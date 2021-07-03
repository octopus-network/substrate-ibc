use crate as pallet_ibc;
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Ibc: pallet_ibc::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

// The ModuleCallbacksImpl creates a static mapping of module index and callback functions of other modules.
// The module index is determined at the time of construct_runtime. For example,
// the index of TemplateModule is 8 in the current runtime.
// In the future, we should find a more dynamic way to create this mapping.
pub struct ModuleCallbacksImpl;

impl pallet_ibc::ModuleCallbacks for ModuleCallbacksImpl {
	fn on_chan_open_try(
		index: usize,
		order: pallet_ibc::ChannelOrder,
		connection_hops: Vec<H256>,
		port_identifier: Vec<u8>,
		channel_identifier: H256,
		counterparty_port_identifier: Vec<u8>,
		counterparty_channel_identifier: H256,
		version: Vec<u8>,
		counterparty_version: Vec<u8>,
	) {
		unimplemented!()
	}

	fn on_chan_open_ack(
		index: usize,
		port_identifier: Vec<u8>,
		channel_identifier: H256,
		version: Vec<u8>,
	) {
		unimplemented!()
	}

	fn on_chan_open_confirm(index: usize, port_identifier: Vec<u8>, channel_identifier: H256) {
		unimplemented!()
	}

	fn on_recv_packet(index: usize, packet: pallet_ibc::Packet) {
		unimplemented!()
	}
}

impl pallet_ibc::Config for Test {
	type Event = Event;
	type ModuleCallbacks = ModuleCallbacksImpl;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
