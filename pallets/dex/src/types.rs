//! Contains all the types for this pallet
#![allow(type_alias_bounds)]

use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::tokens::fungibles::Inspect;
use frame_support::RuntimeDebugNoBound;
use scale_info::TypeInfo;

/// The type identifying a market, which consists of Base and Quote asset
/// e.g.: BTCUSD means BTC is the base asset and is quoted in USD
pub type Market<T: Config> = (AssetIdOf<T>, AssetIdOf<T>);

/// Can either be the Base or Quote asset
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum BaseOrQuote {
	Base,
	Quote,
}

/// Enumerates over buy and sell actions
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum OrderType {
	Buy,
	Sell,
}

/// The balance type used in this crate
pub type BalanceOf<T> =
	<<T as crate::Config>::Currencies as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// The asset id type used in this crate
pub type AssetIdOf<T> =
	<<T as crate::Config>::Currencies as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

/// Contains information about this market
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct MarketInfo<T: Config> {
	/// The balance of the BASE asset in this pool
	pub base_balance: BalanceOf<T>,

	/// The balance of QUOTE asset in this pool
	pub quote_balance: BalanceOf<T>,

	/// The fees collected in this pool, which will be payed out periodically
	pub fees_collected: BalanceOf<T>,
}
