#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
	traits::tokens::fungibles::{Inspect, Transfer},
	transactional, PalletId,
};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type BalanceOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

type AssetIdOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Assets: Inspect<Self::AccountId> + Transfer<Self::AccountId>;

		#[pallet::constant]
		type Eur: Get<AssetIdOf<Self>>;

		#[pallet::constant]
		type Usd: Get<AssetIdOf<Self>>;

		/// The AMM's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LiquidityProvided { who: T::AccountId, eur_amount: BalanceOf<T>, usd_amount: BalanceOf<T> },

		SwappedEurToUsd { who: T::AccountId, eur_amount: BalanceOf<T>, usd_amount: BalanceOf<T> },

		SwappedUsdToEur { who: T::AccountId, eur_amount: BalanceOf<T>, usd_amount: BalanceOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		GenericError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn provide_liquidity(
			origin: OriginFor<T>,
			eur_amount: BalanceOf<T>,
			usd_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let account_id = Self::account_id();

			<T::Assets>::transfer(T::Eur::get(), &who, &account_id, eur_amount, true)?;
			<T::Assets>::transfer(T::Usd::get(), &who, &account_id, usd_amount, true)?;

			Self::deposit_event(Event::LiquidityProvided { who, eur_amount, usd_amount });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn swap_eur_to_usd(origin: OriginFor<T>, eur_amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let account_id = Self::account_id();

			let eur = <T::Assets>::balance(T::Eur::get(), &account_id);
			let usd = <T::Assets>::balance(T::Usd::get(), &account_id);
			let product = eur.checked_mul(&usd).ok_or(Error::<T>::GenericError)?;

			let new_eur = eur.checked_add(&eur_amount).ok_or(Error::<T>::GenericError)?;
			let new_usd = product.checked_div(&new_eur).ok_or(Error::<T>::GenericError)?;

			let usd_amount = usd.checked_sub(&new_usd).ok_or(Error::<T>::GenericError)?;

			<T::Assets>::transfer(T::Eur::get(), &who, &account_id, eur_amount, true)?;
			<T::Assets>::transfer(T::Usd::get(), &account_id, &who, usd_amount, true)?;

			Self::deposit_event(Event::SwappedEurToUsd { who, eur_amount, usd_amount });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn swap_usd_to_eur(origin: OriginFor<T>, usd_amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let account_id = Self::account_id();

			let eur = <T::Assets>::balance(T::Eur::get(), &account_id);
			let usd = <T::Assets>::balance(T::Usd::get(), &account_id);
			let product = eur.checked_mul(&usd).ok_or(Error::<T>::GenericError)?;

			let new_usd = usd.checked_add(&usd_amount).ok_or(Error::<T>::GenericError)?;
			let new_eur = product.checked_div(&new_usd).ok_or(Error::<T>::GenericError)?;

			let eur_amount = eur.checked_sub(&new_eur).ok_or(Error::<T>::GenericError)?;

			<T::Assets>::transfer(T::Usd::get(), &who, &account_id, usd_amount, true)?;
			<T::Assets>::transfer(T::Eur::get(), &account_id, &who, eur_amount, true)?;

			Self::deposit_event(Event::SwappedUsdToEur { who, eur_amount, usd_amount });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the AMM
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
