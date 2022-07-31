//! DEX Pallet
//!
//! This pallet provides functionality for an constant product market maker
//! so x * y = 1, where x and y are the amounts of each currency
//!
//! TODO: more docs

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::Get, BoundedVec, PalletId, RuntimeDebug, RuntimeDebugNoBound};
use frame_system::Origin;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::StaticLookup, AccountId32, Perbill};

use sp_runtime::traits::AccountIdConversion;
use types::*;

mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
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
		/// 0: The market identifier
		PoolCreated(Market<T>),

		/// Emitted when liquidity has been added to a pool
		///
		/// # Fields:
		/// 0: The market identifier for which liquidity has been added
		/// 1: Which asset has been added, either the base or quoted asset
		/// 2: The amount which has been added
		LiquidityAdded(Market<T>, BaseOrQuote, T::Balance),

		/// A liquidity provider (maker) has been rewarded with some balance
		///
		/// # Fields:
		/// 0: The account which received a payout
		/// 1: The amount that has been payed out
		LiquidityProviderRewarded(T::AccountId, T::Balance),

		/// A liquidity taker has swapped an asset for another
		///
		/// # Fields:
		/// 0: The taker account
		/// 1: The market the swap happened on
		/// 2: The asset type that has been swapped out, QUOTE if a sell occurred, BASE if a buy
		/// occurred 3: The amount that was used in the swap
		AssetSwapped(T::AccountId, Market<T>, BaseOrQuote, T::Balance),
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
		/// The user is required to provide both BASE and QUOTE asset
		/// to bootstrap the liquidity of the pool
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// base_asset: The BASE asset of the market
		/// quote_asset: The QUOTE asset of the market
		/// base_amount: Amount of BASE currency to use for bootstrapping liquidity
		/// quote_amount: Amount of QUOTE currency to use for bootstrapping liquidity
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn create_market_pool(
			origin: OriginFor<T>,
			base_asset: T::AssetId,
			quote_asset: T::AssetId,
			base_amount: T::Balance,
			quote_amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// check if market pool exists already
			let market = (base_asset, quote_asset);
			ensure!(Markets::<T>::get(market).is_none(), Error::<T>::MarketExists);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = <pallet_assets::Pallet<T>>::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = <pallet_assets::Pallet<T>>::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// Transfer the assets into the liquidty pool,
			// by using the internal Account
			<pallet_assets::Pallet<T>>::transfer(
				origin,
				base_asset,
				<T::Lookup as StaticLookup>::unlookup(Self::pool_account()),
				base_amount,
			)?;
			// TODO: remember who depsited what

			Markets::<T>::insert(market, MarketInfo::default());

			// Emit the event that the pool has been created
			Self::deposit_event(Event::PoolCreated(market));

			Ok(())
		}

		/// Allows the user to deposit liquidity to a pool,
		/// allowing for rewards to be generated on the deposit.
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// boq: Whether the user deposits the BASE or QUOTE asset
		/// amount: The amount to deposit
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn deposit_liquidity(
			origin: OriginFor<T>,
			boq: BaseOrQuote,
			amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO: update storage

			// TODO: Emit proper event.
			// Self::deposit_event(Event::SomethingStored(something, who));

			Ok(())
		}

		/// Allows the user to withdraw liquidity from a pool
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// boq: Whether the user deposits the BASE or QUOTE asset
		/// amount: The amount to deposit
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn withdraw_liquidity(
			origin: OriginFor<T>,
			bog: BaseOrQuote,
			amount: T::Balance,
		) -> DispatchResult {
			todo!();
			Ok(())
		}

		/// Allows the user to buy the BASE asset of a market
		///
		/// # Arguments
		/// origin: The obiquitous origin of a transaction
		/// market: The market in which the user wants to trade
		/// amount: The amount of the QUOTE asset the user is willing to spend
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
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn sell(origin: OriginFor<T>, market: Market<T>, amount: T::Balance) -> DispatchResult {
			todo!();
			Ok(())
		}

		/// Allows the user to get a fill price estimate for a given market and desired amount
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: The market for which the price estimate is emitted
		/// buy_or_sell: Whether the user wants a buy or sell estimate
		/// amount: The amount dictates the slippage and price impact
		#[pallet::weight(1_000)]
		pub fn fill_price_estimate(
			origin: OriginFor<T>,
			market: Market<T>,
			buy_or_sell: BuyOrSell,
			amount: T::Balance,
		) -> DispatchResult {
			todo!();
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The internal account of the pool derived from this pallets id
	pub fn pool_account() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}
}
