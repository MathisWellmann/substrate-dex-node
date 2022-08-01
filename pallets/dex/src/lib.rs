//! DEX Pallet
//!
//! This pallet provides functionality for an constant product market maker
//! Similar to Uniswap v2.
//!
//! TODO: more docs

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{traits::Get, transactional, PalletId};
pub use pallet::*;
use pallet_assets::WeightInfo;
use sp_arithmetic::traits::*;
use sp_runtime::{
	traits::{Saturating, StaticLookup, Zero},
	DispatchError,
};

use sp_runtime::traits::AccountIdConversion;
use types::*;

mod types;

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

		/// The taker fee a user pays for taking liquidity and doing the asset swap
		/// First item is the numerator, second one the denominator
		/// fee_rate = numerator / denominator.
		#[pallet::constant]
		type TakerFee: Get<(u32, u32)>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores information about the markets liquidity pool
	///
	/// Maps Market => (BASE Balance, QUOTE Balance)
	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPool<T: Config> =
		StorageMap<_, Blake2_128Concat, Market<T>, (T::Balance, T::Balance), OptionQuery>;

	/// Stores information regarding the liquidity provision of users in a given market
	/// Used for rewarding liquidity providers from the collected taker fees.
	///
	/// Maps Market and Account => (BASE Balance, QUOTE Balance)
	#[pallet::storage]
	#[pallet::getter(fn liq_provision_pool)]
	pub type LiqProvisionPool<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Market<T>,
		Blake2_128Concat,
		T::AccountId,
		(T::Balance, T::Balance),
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A liquidity pool has been created for a trading pair
		///
		/// # Fields:
		/// 0: Who created the market
		/// 1: The market identifier
		/// 2: Liquidity for BASE asset
		/// 3: Liquidity for QUOTE asset
		PoolCreated(T::AccountId, Market<T>, T::Balance, T::Balance),

		/// Emitted when liquidity has been added to a pool
		///
		/// # Fields:
		/// 0: The liquidity provider account
		/// 1: The market identifier for which liquidity has been added
		/// 2: The BASE asset balance added
		/// 3: The QUOT asset balance added
		LiquidityAdded(T::AccountId, Market<T>, T::Balance, T::Balance),

		/// Emitted when a user removes liquidity from a pool
		///
		/// # Fields:
		/// 0: The account withdrawing the liquidity
		/// 1: The market it's been withdrawn from
		/// 2: The amount of BASE asset withdrawn
		/// 3: The amount of QUOTE asset withdrawn
		LiquidtyWithdrawn(T::AccountId, Market<T>, T::Balance, T::Balance),

		/// A liquidity provider (maker) has been rewarded with some balance
		///
		/// # Fields:
		/// 0: The account which received a payout
		/// 1: The amount that has been payed out
		LiquidityProviderRewarded(T::AccountId, T::Balance),

		/// A user bought the BASE asset
		///
		/// # Fields:
		/// 0: The account which bought
		/// 1: The market in which it was bough
		/// 2: The amount of QUOTE asset that was spent
		/// 3: The amount of BASE asset received
		Bought(T::AccountId, Market<T>, T::Balance, T::Balance),

		/// A user sold the BASE asset
		///
		/// # Fields:
		/// 0: The account which sold
		/// 1: The market in which it was sold
		/// 2: The amount of BASE asset that was sold
		/// 3: The amount of QUOTE asset received
		Sold(T::AccountId, Market<T>, T::Balance, T::Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The market already exists and cannot be created
		MarketExists,

		/// The market the user specified does not exist
		MarketDoesNotExist,

		/// The user does not have enough balance
		NotEnoughBalance,

		/// Some arithmetic error occurred
		ArithmeticError,
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
		///
		/// # Weight:
		/// Requires base weight + 3 reads and 2 writes, as well as two times the weight of transfer operation of assets pallet
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 2) + <T as pallet_assets::Config>::WeightInfo::transfer())]
		#[transactional] // This Dispatchable is atomic
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
			ensure!(LiquidityPool::<T>::get(market).is_none(), Error::<T>::MarketExists);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = <pallet_assets::Pallet<T>>::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = <pallet_assets::Pallet<T>>::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// Transfer the assets into the liquidty pool,
			// by using the internal Account
			<pallet_assets::Pallet<T>>::transfer(
				origin.clone(),
				base_asset,
				<T::Lookup as StaticLookup>::unlookup(Self::pool_account()),
				base_amount,
			)?;
			<pallet_assets::Pallet<T>>::transfer(
				origin,
				quote_asset,
				<T::Lookup as StaticLookup>::unlookup(Self::pool_account()),
				quote_amount,
			)?;

			// Insert the balance information for the market
			LiquidityPool::<T>::insert(market, (base_amount, quote_amount));

			// remember who depsited what in the liquidity provision pool
			LiqProvisionPool::<T>::insert(market, who.clone(), (base_amount, quote_amount));

			// Emit the event that the pool has been created
			Self::deposit_event(Event::PoolCreated(who, market, base_amount, quote_amount));

			Ok(())
		}

		/// Allows the user to deposit liquidity to a pool,
		/// allowing for rewards to be generated on the deposit.
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: To which market the liquidity should be added
		/// base_amount: The amount of BASE asset to deposit
		/// quote_amount: The amount of QUOTE asset to deposit
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional] // This Dispatchable is atomic
		pub fn deposit_liquidity(
			origin: OriginFor<T>,
			market: Market<T>,
			base_amount: T::Balance,
			quote_amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let (base_asset, quote_asset) = market;

			// check if market pool exists
			ensure!(LiquidityPool::<T>::get(market).is_some(), Error::<T>::MarketDoesNotExist);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = <pallet_assets::Pallet<T>>::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = <pallet_assets::Pallet<T>>::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// Use try_mutate in case the closure fails, e.g.: arithmetic overflow
			LiquidityPool::<T>::try_mutate(market, |opt_balances| -> DispatchResult {
				let (base_balance, quote_balance) = opt_balances
					.expect("Check that the market pool exists has been done before; qed");

				base_balance.checked_add(&base_amount).ok_or(Error::<T>::ArithmeticError)?;
				quote_balance.checked_add(&quote_amount).ok_or(Error::<T>::ArithmeticError)?;

				Ok(())
			})?;

			// transfer the BASE asset to pool account
			<pallet_assets::Pallet<T>>::transfer(
				origin.clone(),
				base_asset,
				<T::Lookup as StaticLookup>::unlookup(Self::pool_account()),
				base_amount,
			)?;
			// transfer the QUOTE asset to pool account
			<pallet_assets::Pallet<T>>::transfer(
				origin,
				quote_asset,
				<T::Lookup as StaticLookup>::unlookup(Self::pool_account()),
				quote_amount,
			)?;

			// Keep track of liquidity providers
			LiqProvisionPool::<T>::try_mutate(
				market,
				who.clone(),
				|opt_balances| -> DispatchResult {
					let (base_balance, quote_balance) = opt_balances
						.expect("The existance of the balances here has been checked before; qed");

					base_balance.checked_add(&base_amount).ok_or(Error::<T>::ArithmeticError)?;
					quote_balance.checked_add(&quote_amount).ok_or(Error::<T>::ArithmeticError)?;

					Ok(())
				},
			)?;

			Self::deposit_event(Event::LiquidityAdded(who, market, base_amount, quote_amount));

			Ok(())
		}

		/// Allows the user to withdraw liquidity from a pool
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// boq: Whether the user deposits the BASE or QUOTE asset
		/// amount: The amount to deposit
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
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
		/// quote_amount: The amount of the QUOTE asset the user is willing to spend
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
		pub fn buy(
			origin: OriginFor<T>,
			market: Market<T>,
			quote_amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// get balance of pool, if it exists
			let (pool_base_balance, pool_quote_balance) =
				LiquidityPool::<T>::get(market).ok_or(Error::<T>::MarketDoesNotExist)?;

			let (base_asset, quote_asset) = market;

			// Check that balance of QUOTE asset of caller account is sufficient
			let quote_balance = <pallet_assets::Pallet<T>>::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// get the amount to receive
			let receive_amount = Self::get_received_amount(
				pool_base_balance,
				pool_quote_balance,
				BuyOrSell::Buy,
				quote_amount,
			)?;

			let pool_account = <T::Lookup as StaticLookup>::unlookup(Self::pool_account());

			// Transfer the QUOTE asset into the pool
			<pallet_assets::Pallet<T>>::transfer(
				origin,
				quote_asset,
				pool_account.clone(),
				quote_amount,
			)?;
			// And get the BASE asset out of the pool
			<pallet_assets::Pallet<T>>::force_transfer(
				frame_system::RawOrigin::Root.into(),
				base_asset,
				pool_account,
				<T::Lookup as StaticLookup>::unlookup(who.clone()),
				receive_amount,
			)?;

			// TODO: collect fees somewhere

			Self::deposit_event(Event::Bought(who, market, quote_amount, receive_amount));

			Ok(())
		}

		/// Allows the user to sell the BASE asset of a market
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: The market in which the user wants to trade
		/// base_amount: The amount of BASE asset the user wants to sell
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
		pub fn sell(
			origin: OriginFor<T>,
			market: Market<T>,
			base_amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// get balance of pool, if it exists
			let (pool_base_balance, pool_quote_balance) =
				LiquidityPool::<T>::get(market).ok_or(Error::<T>::MarketDoesNotExist)?;

			let (base_asset, quote_asset) = market;

			// Check that user has enough BASE asset to sell it
			let base_balance = <pallet_assets::Pallet<T>>::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			let receive_amount = Self::get_received_amount(
				pool_base_balance,
				pool_quote_balance,
				BuyOrSell::Sell,
				base_amount,
			)?;

			let pool_account = <T::Lookup as StaticLookup>::unlookup(Self::pool_account());

			// Transfer the BASE asset into the pool
			<pallet_assets::Pallet<T>>::transfer(origin, base_asset, pool_account, base_amount)?;
			// And get the QUOTE asset out of the pool
			<pallet_assets::Pallet<T>>::transfer(
				frame_system::RawOrigin::Root.into(),
				quote_asset,
				<T::Lookup as StaticLookup>::unlookup(who.clone()),
				receive_amount,
			)?;

			// TODO: collect fees somewhere

			Self::deposit_event(Event::Sold(who, market, base_amount, receive_amount));

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
	#[inline(always)]
	pub fn pool_account() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Calculates the received amount when buying or selling a given amount
	///
	/// # Arguments:
	/// pool_base_balance: The amount of the BASE asset in the pool
	/// pool_quote_balance: The amount of the QUOTE asset in the pool
	/// buy_or_sell: Whether the operation is buying or selling
	/// amount: The amount to spend
	///
	/// # Returns:
	/// If Ok, The balance that the user will receive from this exchange
	/// Else some arithmetic error
	pub fn get_received_amount(
		pool_base_balance: T::Balance,
		pool_quote_balance: T::Balance,
		buy_or_sell: BuyOrSell,
		amount: T::Balance,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			Ok(Zero::zero())
		} else {
			let (fee_numerator, fee_denominator) = T::TakerFee::get();

			// TODO: match on buy_or_sell

			let supply_with_fee = amount
				.saturating_mul(T::Balance::from(fee_denominator.saturating_sub(fee_numerator)));
			let numerator = supply_with_fee.saturating_mul(pool_base_balance);
			let denom = pool_quote_balance
				.saturating_mul(T::Balance::from(fee_denominator))
				.saturating_add(supply_with_fee);

			let receive_amount = numerator
				.checked_div(&T::Balance::from(denom))
				.ok_or(Error::<T>::ArithmeticError)?;

			Ok(receive_amount)
		}
	}
}
