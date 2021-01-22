//! # Pesa Module
//!
//! - [`pesa::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! Pesa helps to assign phone number to an address and provide phone number to address resolution.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `register` - Register phone number with an account.
//! * `resolve` - Perform phone number to address resolution.
//!
//! [`Call`]: ./enum.Call.html
//! [`Trait`]: ./trait.Trait.html

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::{
	traits::{StaticLookup, Zero}
};
use frame_support::{
	decl_module, decl_event, decl_storage, ensure, decl_error,
	traits::{Currency, EnsureOrigin, ReservableCurrency, OnUnbalanced, Get},
};
use frame_system::ensure_signed;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The currency trait.
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Reservation fee.
	type ReservationFee: Get<BalanceOf<Self>>;

	/// What to do with slashed funds.
	type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// The origin which may forcibly set or remove a number. Root can always do this.
	type ForceOrigin: EnsureOrigin<Self::Origin>;

	/// The minimum length a phone number can be.
	type MinLength: Get<usize>;

	/// The maximum length a phone number can be.
	type MaxLength: Get<usize>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Pesa {
		/// The lookup table for wallet address.
		NumberOf: map hasher(twox_64_concat) T::AccountId => Option<(Vec<u8>, BalanceOf<T>)>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, Balance = BalanceOf<T> {
		/// Phone number was registered with an address. \[who, fee\]
		NumberSet(AccountId),
		NumberChanged(AccountId),
		NumberCleared(AccountId, Balance),
	}
);

decl_error! {
	/// Error for the Pesa module.
	pub enum Error for Module<T: Trait> {
		/// Phone number is too short.
		TooShort,
		/// Phone number is too long.
		TooLong,
		/// An account isn't registered.
		UnRegistered,
	}
}

decl_module! {
	/// Pesa module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Reservation fee.
		const ReservationFee: BalanceOf<T> = T::ReservationFee::get();

		/// The minimum length a name may be.
		const MinLength: u32 = T::MinLength::get() as u32;

		/// The maximum length a name may be.
		const MaxLength: u32 = T::MaxLength::get() as u32;

		/// Set an account's phone number. The number should be a UTF-8-encoded string with country code.
		///
		/// The name may not be more than `T::MaxLength` bytes, nor less than `T::MinLength` bytes.
		///
		/// If the account doesn't already have a number, then a fee of `ReservationFee` is reserved
		/// in the account.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - At most one balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[weight = 50_000_000]
		fn set_phone_number(origin, phone_number: Vec<u8>) {
			let sender = ensure_signed(origin)?;

			ensure!(phone_number.len() >= T::MinLength::get(), Error::<T>::TooShort);
			ensure!(phone_number.len() <= T::MaxLength::get(), Error::<T>::TooLong);

			let deposit = if let Some((_, deposit)) = <NumberOf<T>>::get(&sender) {
				Self::deposit_event(RawEvent::NumberChanged(sender.clone()));
				deposit
			} else {
				let deposit = T::ReservationFee::get();
				T::Currency::reserve(&sender, deposit.clone())?;
				Self::deposit_event(RawEvent::NumberSet(sender.clone()));
				deposit
			};

			<NumberOf<T>>::insert(&sender, (phone_number, deposit));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::{
		assert_ok, assert_noop, impl_outer_origin, parameter_types, weights::Weight,
		ord_parameter_types
	};
	use sp_core::H256;
	use frame_system::EnsureSignedBy;
	use sp_runtime::{
		Perbill, testing::Header, traits::{BlakeTwo256, IdentityLookup, BadOrigin},
	};

	impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}
	impl frame_system::Trait for Test {
		type BaseCallFilter = ();
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Call = ();
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = ();
		type BlockExecutionWeight = ();
		type ExtrinsicBaseWeight = ();
		type MaximumExtrinsicWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type PalletInfo = ();
		type AccountData = pallet_balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
	}
	parameter_types! {
		pub const ExistentialDeposit: u64 = 1;
	}
	impl pallet_balances::Trait for Test {
		type MaxLocks = ();
		type Balance = u64;
		type Event = ();
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
	}
	parameter_types! {
		pub const ReservationFee: u64 = 2;
		pub const MinLength: usize = 3;
		pub const MaxLength: usize = 16;
	}
	ord_parameter_types! {
		pub const One: u64 = 1;
	}
	impl Trait for Test {
		type Event = ();
		type Currency = Balances;
		type ReservationFee = ReservationFee;
		type Slashed = ();
		type ForceOrigin = EnsureSignedBy<One, u64>;
		type MinLength = MinLength;
		type MaxLength = MaxLength;
	}
	type System = frame_system::Module<Test>;
	type Balances = pallet_balances::Module<Test>;
	type Pesa = Module<Test>;

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(1, 10),
				(2, 10),
			],
		}.assimilate_storage(&mut t).unwrap();
		t.into()
	}

	#[test]
	fn register_phone_number_should_work() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pesa::register(Origin::signed(1), b"Jack".to_vec()));
			assert_eq!(Balances::reserved_balance(1), 2);
			assert_eq!(Balances::free_balance(1), 8);
			assert_eq!(<NameOf<Test>>::get(1).unwrap().0, b"Gav".to_vec());
		});
	}
}
