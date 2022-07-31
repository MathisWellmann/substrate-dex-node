//! Contains all the types for this pallet

use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{RuntimeDebug, RuntimeDebugNoBound};
use scale_info::TypeInfo;

/// The type identifying a market, which consists of Base and Quote asset
/// e.g.: BTCUSD means BTC is the base asset and is quoted in USD
pub type Market<T: Config> =
	(<T as pallet_assets::Config>::AssetId, <T as pallet_assets::Config>::AssetId);

/// Can either be the Base or Quote asset
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum BaseOrQuote {
	Base,
	Quote,
}

/// Enumerates over buy and sell actions
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum BuyOrSell {
	Buy,
	Sell,
}
