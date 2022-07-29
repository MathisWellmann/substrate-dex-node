#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
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
	#[pallet::getter(fn markets)]
	pub type Markets<T: Config> =
		StorageMap<_, Blake2_128Concat, Market<T>, MarketInfo, OptionQuery>;

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

		/// A liquidity provider (maker) has been rewarded with some balance
		///
		/// # Fields:
		/// 0: The account which received a payout
		/// 1: The amount that has been payed out
		///
		LiquidityProviderRewarded(T::AccountId, T::Balance),

		/// A liquidity taker has swapped an asset for another
		///
		/// # Fields:
		/// 0: The taker account
		/// 1: The market the swap happened on
		/// 2: TODO:
		/// 3: The amount that was used in the swap
		///
		AssetSwapped(T::AccountId, Market<T>, BaseOrQuote<T>, T::Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The market already exists and cannot be created
		MarketExists,

		/// The market the user specified does not exist
		MarketDoesNotExist,

		/// The user does not have enough balance
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new pool for a market if it does not exist already
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// base_asset: The BASE asset of the market
		/// quote_asset: The QUOTE asset of the market
		///
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn create_market_pool(
			origin: OriginFor<T>,
			base_asset: StringProxy<T>,
			quote_asset: StringProxy<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO:

			Ok(())
		}

		/// Allows the user to deposit liquidity to a pool,
		/// allowing for rewards to be generated on the deposit.
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// boq: Whether the user deposits the BASE or QUOTE asset
		/// amount: The amount to deposit
		///
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn deposit_to_pool(
			origin: OriginFor<T>,
			boq: BaseOrQuote<T>,
			amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO: update storage

			// TODO: Emit proper event.
			// Self::deposit_event(Event::SomethingStored(something, who));

			Ok(())
		}

		/// Allows the user to buy the BASE asset of a market
		///
		/// # Arguments
		/// origin: The obiquitous origin of a transaction
		/// market: The market in which the user wants to trade
		/// amount: The amount of the QUOTE asset the user is willing to spend
		///
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn buy(origin: OriginFor<T>, market: Market<T>, amount: T::Balance) -> DispatchResult {
			todo!();
			Ok(())
		}

		/// Allows the user to sell the BASE asset of a market
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: The market in which the user wants to trade
		/// amount: The amount of BASE asset the user wants to sell
		///
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn sell(origin: OriginFor<T>, market: Market<T>, amount: T::Balance) -> DispatchResult {
			todo!();
			Ok(())
		}
	}
}
