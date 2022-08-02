//! DEX Pallet
//!
//! This pallet provides functionality for an constant product market maker
//! Similar to Uniswap v2.
//!
//! # Overview:
//! A decentralized automated market maker allows users to buy and sell
//! assets where the counterparty is the liqudity pool itself
//! rather than another user (unlike orderbook based exchanges),
//! This has the advantage of "always on" liquidity.
//! This is achieved by keeping the value of the liquidity pool constant,
//! so only the prices change as the quantity changes
//!
//! # Interface:
//! create_market_pool: Allows the user to create a liquidity pool with some initial balance
//! deposit_liquidity: Allows the user to add liqudity to a pool to earn part of the collected fees
//! withdraw_liquidity: Allows the user to remove his liquidity from a pool
//! buy: Allows the user to exchange the QUOTE asset for the BASE asset
//! sell: Allows the user to exchange the BASE asset for the QUOTE asset
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

use frame_support::{
	traits::{
		tokens::fungibles::{Inspect, Transfer},
		Get,
	},
	transactional, PalletId,
};
pub use pallet::*;
use sp_arithmetic::traits::*;
use sp_runtime::{traits::Zero, DispatchError};

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
	pub trait Config: frame_system::Config {
		/// The ubiqutous event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The taker fee a user pays for taking liquidity and doing the asset swap
		/// First item is the numerator, second one the denominator
		/// fee_rate = numerator / denominator.
		#[pallet::constant]
		type TakerFee: Get<(u32, u32)>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The type that enables currency transfers
		type Currencies: Transfer<Self::AccountId, Balance = u128, AssetId = u8>;
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
		StorageMap<_, Blake2_128Concat, Market<T>, (BalanceOf<T>, BalanceOf<T>), OptionQuery>;

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
		(BalanceOf<T>, BalanceOf<T>),
		ValueQuery,
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
		PoolCreated(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// Emitted when liquidity has been added to a pool
		///
		/// # Fields:
		/// 0: The liquidity provider account
		/// 1: The market identifier for which liquidity has been added
		/// 2: The BASE asset balance added
		/// 3: The QUOT asset balance added
		LiquidityAdded(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// Emitted when a user removes liquidity from a pool
		///
		/// # Fields:
		/// 0: The account withdrawing the liquidity
		/// 1: The market it's been withdrawn from
		/// 2: The amount of BASE asset withdrawn
		/// 3: The amount of QUOTE asset withdrawn
		LiquidtyWithdrawn(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// A liquidity provider (maker) has been rewarded with some balance
		///
		/// # Fields:
		/// 0: The account which received a payout
		/// 1: The amount that has been payed out
		LiquidityProviderRewarded(T::AccountId, BalanceOf<T>),

		/// A user bought the BASE asset
		///
		/// # Fields:
		/// 0: The account which bought
		/// 1: The market in which it was bough
		/// 2: The amount of QUOTE asset that was spent
		/// 3: The amount of BASE asset received
		Bought(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// A user sold the BASE asset
		///
		/// # Fields:
		/// 0: The account which sold
		/// 1: The market in which it was sold
		/// 2: The amount of BASE asset that was sold
		/// 3: The amount of QUOTE asset received
		Sold(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),
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
		/// Requires base weight + 3 reads and 2 writes
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 2))]
		#[transactional] // This Dispatchable is atomic
		pub fn create_market_pool(
			origin: OriginFor<T>,
			base_asset: AssetIdOf<T>,
			quote_asset: AssetIdOf<T>,
			base_amount: BalanceOf<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// check if market pool exists already
			let market = (base_asset, quote_asset);
			ensure!(LiquidityPool::<T>::get(market).is_none(), Error::<T>::MarketExists);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = Self::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = Self::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			let pool_account = Self::pool_account();

			// Transfer the BASE currency into the pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_account,
				base_amount,
				true,
			)?;
			// Transfer the QUOTE currency into the pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_account,
				quote_amount,
				true,
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
			base_amount: BalanceOf<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let (base_asset, quote_asset) = market;

			// check if market pool exists
			ensure!(LiquidityPool::<T>::get(market).is_some(), Error::<T>::MarketDoesNotExist);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = Self::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = Self::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// Use try_mutate in case the closure fails, e.g.: arithmetic overflow
			LiquidityPool::<T>::try_mutate(market, |opt_balances| -> DispatchResult {
				let (base_balance, quote_balance) = opt_balances
					.expect("Check that the market pool exists has been done before; qed");

				base_balance.checked_add(base_amount).ok_or(Error::<T>::ArithmeticError)?;
				quote_balance.checked_add(quote_amount).ok_or(Error::<T>::ArithmeticError)?;

				Ok(())
			})?;

			let pool_account = Self::pool_account();

			// transfer the BASE currency to pool account
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_account,
				base_amount,
				true,
			)?;
			// transfer the QUOTE currency to pool account
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_account,
				quote_amount,
				true,
			)?;

			// Keep track of liquidity providers
			LiqProvisionPool::<T>::try_mutate(
				market,
				who.clone(),
				|(base_balance, quote_balance)| -> DispatchResult {
					*base_balance =
						base_balance.checked_add(base_amount).ok_or(Error::<T>::ArithmeticError)?;
					*quote_balance = quote_balance
						.checked_add(quote_amount)
						.ok_or(Error::<T>::ArithmeticError)?;

					Ok(())
				},
			)?;

			Self::deposit_event(Event::LiquidityAdded(who, market, base_amount, quote_amount));

			Ok(())
		}

		/// Allows the user to withdraw his liquidity from a pool
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: The liquidity pool to withdraw from
		/// base_amount: The amount of the BASE asset to withdraw
		/// quote_amount: The amount of the QUOTE asset to withdraw
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
		pub fn withdraw_liquidity(
			origin: OriginFor<T>,
			market: Market<T>,
			base_amount: BalanceOf<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let (base_asset, quote_asset) = market;
			let pool_account = Self::pool_account();

			// ensure the user has enough balance in the pool to withdraw
			let (users_base_balance, users_quote_balance) =
				LiqProvisionPool::<T>::get(market, &who);
			ensure!(users_base_balance >= base_amount, Error::<T>::NotEnoughBalance);
			ensure!(users_quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// transfer out BASE asset from pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&pool_account,
				&who,
				base_amount,
				true,
			)?;
			// transfer out QUOTE asset from pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&pool_account,
				&who,
				quote_amount,
				true,
			)?;

			// update LiqProvisionPool
			LiqProvisionPool::<T>::try_mutate(
				market,
				who.clone(),
				|(base_balance, quote_balance)| -> DispatchResult {
					*base_balance =
						base_balance.checked_sub(base_amount).ok_or(Error::<T>::ArithmeticError)?;
					*quote_balance = quote_balance
						.checked_sub(quote_amount)
						.ok_or(Error::<T>::ArithmeticError)?;

					Ok(())
				},
			)?;

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
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// get balance of pool, if it exists
			let (pool_base_balance, pool_quote_balance) =
				LiquidityPool::<T>::get(market).ok_or(Error::<T>::MarketDoesNotExist)?;

			let (base_asset, quote_asset) = market;

			// Check that balance of QUOTE asset of caller account is sufficient
			let quote_balance = Self::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// get the amount to receive
			let receive_amount = Self::get_received_amount(
				pool_base_balance,
				pool_quote_balance,
				OrderType::Buy,
				quote_amount,
			)?;

			let pool_account = Self::pool_account();

			// Transfer the QUOTE asset into the pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_account,
				quote_amount,
				true,
			)?;
			// And get the BASE asset out of the pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&pool_account,
				&who,
				receive_amount,
				true,
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
			base_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// get balance of pool, if it exists
			let (pool_base_balance, pool_quote_balance) =
				LiquidityPool::<T>::get(market).ok_or(Error::<T>::MarketDoesNotExist)?;

			let (base_asset, quote_asset) = market;

			// Check that user has enough BASE asset to sell it
			let base_balance = Self::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			let receive_amount = Self::get_received_amount(
				pool_base_balance,
				pool_quote_balance,
				OrderType::Sell,
				base_amount,
			)?;

			let pool_account = Self::pool_account();

			// Transfer the BASE asset into the pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_account,
				base_amount,
				true,
			)?;
			// And get the QUOTE asset out of the pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&pool_account,
				&who,
				receive_amount,
				true,
			)?;

			// TODO: collect fees somewhere

			Self::deposit_event(Event::Sold(who, market, base_amount, receive_amount));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The internal account of the pool derived from this pallets id
	#[inline(always)]
	fn pool_account() -> T::AccountId {
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
	fn get_received_amount(
		pool_base_balance: BalanceOf<T>,
		pool_quote_balance: BalanceOf<T>,
		buy_or_sell: OrderType,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		if amount.is_zero() {
			Ok(Zero::zero())
		} else {
			let pool_k = pool_base_balance
				.checked_mul(pool_quote_balance)
				.ok_or(Error::<T>::ArithmeticError)?;

			// TODO: include fees

			let receive_amount = match buy_or_sell {
				OrderType::Buy => {
					let new_quote_balance = pool_quote_balance
						.checked_add(amount)
						.ok_or(Error::<T>::ArithmeticError)?;
					let new_base_balance =
						pool_k.checked_div(new_quote_balance).ok_or(Error::<T>::ArithmeticError)?;
					pool_base_balance
						.checked_sub(new_base_balance)
						.ok_or(Error::<T>::ArithmeticError)?
				},
				OrderType::Sell => {
					let new_base_balance =
						pool_base_balance.checked_add(amount).ok_or(Error::<T>::ArithmeticError)?;
					let new_quote_balance =
						pool_k.checked_div(new_base_balance).ok_or(Error::<T>::ArithmeticError)?;
					pool_quote_balance
						.checked_sub(new_quote_balance)
						.ok_or(Error::<T>::ArithmeticError)?
				},
			};

			Ok(receive_amount)
		}
	}

	/// Helper function to get the account balance easily
	///
	/// # Arguments:
	/// asset_id: The asset were trying to query
	/// who: The account for which the balance should be retrived
	///
	/// # Returns:
	/// The balance of a user for a given asset
	fn balance(asset_id: AssetIdOf<T>, who: &T::AccountId) -> BalanceOf<T> {
		<<T as Config>::Currencies as Inspect<<T as frame_system::Config>::AccountId>>::balance(
			asset_id, who,
		)
	}
}
