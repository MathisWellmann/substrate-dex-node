#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait DexRuntimeApi {
		/// Gets the current price for a market
		///
		/// # Arguments:
		/// market: (BASE AssetId, QUOTE AssetId)
		///
		/// # Returns:
		/// The current price of the market
		/// represented as (numerator, denominator)
		fn current_price(market: (u8, u8)) -> (u128, u128);
	}
}
