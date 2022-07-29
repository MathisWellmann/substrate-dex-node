#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{BoundedVec, RuntimeDebug, RuntimeDebugNoBound};
pub use pallet::*;

use scale_info::TypeInfo;
use sp_runtime::Perbill;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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
pub enum BaseOrQuote<T: Config> {
	Base(BaseAsset<T>),
	Quote(QuoteAsset<T>),
}

/// Basically valid utf-8 bytes, just like a String, but bounded in size
/// The String limit bound is coming from pallet_assets
pub type StringProxy<T: Config> = BoundedVec<u8, T::StringLimit>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// The taker fee a user pays for taking liquidity and doing the asset swap
		type TakerFee: Get<Perbill>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A liquidity pool has been created for a trading pair
		///
		/// # Fields:
		/// 0: Base asset
		/// 1: Quote asset
		///
		PoolCreated(BaseAsset<T>, QuoteAsset<T>),

		/// Emitted when liquidity has been added to a pool
		///
		/// # Fields:
		/// 0: The market identifier for which liquidity has been added
		/// 1: Which asset has been added, either the base or quoted asset
		/// 2: The amount which has been added
		///
		LiquidityAdded(Market<T>, BaseOrQuote<T>, T::Balance),
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn create_pool(
			origin: OriginFor<T>,
			base_asset: StringProxy<T>,
			quote_asset: StringProxy<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO:

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn deposit_to_pool(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO: update storage

			// TODO: Emit proper event.
			// Self::deposit_event(Event::SomethingStored(something, who));

			Ok(())
		}
	}
}
