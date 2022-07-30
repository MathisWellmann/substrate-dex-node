//! Contains all the types for this pallet

use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec, RuntimeDebug, RuntimeDebugNoBound};
use scale_info::TypeInfo;

/// The asset to be exchanged against the quoted asset
/// Defined for readability and clarity
pub type BaseAsset<T: Config> = StringProxy<T>;

/// The asset used for quoting this market
/// Defined for readability and clarity
pub type QuoteAsset<T: Config> = StringProxy<T>;

/// The type identifying a market, which consists of Base and Quote asset
/// e.g.: BTCUSD means BTC is the base asset and is quoted in USD
pub type Market<T: Config> = (BaseAsset<T>, QuoteAsset<T>);

/// Can either be the Base or Quote asset
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum BaseOrQuote<T: Config> {
	Base(BaseAsset<T>),
	Quote(QuoteAsset<T>),
}

/// Basically valid utf-8 bytes, just like a String, but bounded in size
/// The String limit bound is coming from pallet_assets
pub type StringProxy<T: Config> = BoundedVec<u8, T::StringLimit>;

/// Contains information about a market in addition to BASE and QUOTE assets
#[derive(RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MarketInfo {}

/// Enumerates over buy and sell actions
#[derive(RuntimeDebugNoBound, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum BuyOrSell {
	Buy,
	Sell,
}
