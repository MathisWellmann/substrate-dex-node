use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
	PalletId,
};
use frame_system::EnsureRoot;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	AccountId32, BuildStorage, MultiSignature,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type BlockNumber = u64;
pub type Signature = MultiSignature;
// pub type AccountId = u8;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type Index = u64;
pub type Hash = sp_core::H256;
pub type AssetId = u8;

pub const ALICE: AccountId = AccountId32::new([0; 32]);
pub const BOB: AccountId = AccountId32::new([1; 32]);
pub const CHARLIE: AccountId = AccountId32::new([2; 32]);
pub const EMPTY_ACCOUNT: AccountId = AccountId32::new([3; 32]);
pub const DEX_PALLET_ACCOUNT: AccountId = AccountId32::new([
	109, 111, 100, 108, 100, 101, 120, 112, 97, 108, 108, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
]);

pub const BTC: AssetId = 0;
pub const XMR: AssetId = 1;
pub const DOT: AssetId = 2;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Dex: crate,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<500>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

parameter_types! {
	pub const AssetDeposit: u128 = 1;
	pub const AssetAccountDeposit: u128 = 1;
	pub const MetadataDepositBase: u128 = 1;
	pub const MetadataDepositPerByte: u128 = 1;
	pub const ApprovalDeposit: u128 = 1;
	/// The assets should be identified with up to 6 characters
	/// such as BTC, ETH or XMR
	pub const StringLimit: u8 = 6;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

parameter_types! {
	// 10 Basis points taker fee, which is lower vs uniswap but may attract more taker flow
	pub TakerFee: (u32, u32) = (1, 1_000);
	// Only 8 bytes available, so t is missing at the end
	pub DexPalletId: PalletId = PalletId(*b"dexpalle");
}

impl crate::Config for Test {
	type Event = Event;
	type TakerFee = TakerFee;
	type PalletId = DexPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	GenesisConfig {
		balances: BalancesConfig {
			balances: vec![
				(ALICE, 1_000_000_000_000),
				(BOB, 1_000_000_000_000),
				(CHARLIE, 1_000_000_000_000),
			],
		},
		assets: AssetsConfig {
			assets: vec![
				(BTC, DEX_PALLET_ACCOUNT, true, 1),
				(XMR, DEX_PALLET_ACCOUNT, true, 1),
				(DOT, DEX_PALLET_ACCOUNT, true, 1),
			],
			metadata: vec![],
			accounts: vec![
				(BTC, ALICE, 1_000_000_000),
				(XMR, ALICE, 1_000_000_000),
				(DOT, ALICE, 1_000_000_000),
				(BTC, BOB, 1_000_000_000),
				(BTC, CHARLIE, 1_000_000_000),
			],
		},
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	// Set the block number to 1 as genesis Events are not captured
	ext.execute_with(|| System::set_block_number(1));

	ext
}
