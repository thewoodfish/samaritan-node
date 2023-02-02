#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxUriLength: Get<u32>;

		#[pallet::constant]
		type MaxKeyLength: Get<u32>;
	}

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn ht)]
	pub type HashTableUri<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxUriLength>, ValueQuery>;

	#[pallet::storage]
	pub(super) type FragRecord<T: Config> = StorageDoubleMap<
		_, 
		Blake2_128Concat, T::AccountId, 
		Blake2_128Concat, BoundedVec<u8, T::MaxKeyLength>, 
		(bool, BoundedVec<u8, T::MaxUriLength>), 
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		HashTableUriUpdate { uri: Vec<u8> },
		NewRecordEntry { key: Vec<u8>, uri: Vec<u8> },
		RecordEntryDeleted { key: Vec<u8> }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		UriOverflow,
		KeyOverflow
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn update_hashtable_uri(origin: OriginFor<T>, cid: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let mut uri: BoundedVec<_, T::MaxUriLength> = Default::default();

			for i in cid.clone() { uri.try_push(i).map_err(|_| Error::<T>::UriOverflow)? };

			// set the latest root uri
			HashTableUri::<T>::put(uri);
			Self::deposit_event(Event::HashTableUriUpdate { uri: cid });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn record_data_entry(origin: OriginFor<T>, ekey: Vec<u8>, cid: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut uri: BoundedVec<_, T::MaxUriLength> = Default::default();
			let mut key: BoundedVec<_, T::MaxKeyLength> = Default::default();

			for i in cid.clone() { uri.try_push(i).map_err(|_| Error::<T>::UriOverflow)? };
			for j in ekey.clone() { key.try_push(j).map_err(|_| Error::<T>::KeyOverflow)? };

			// insert into storage
			FragRecord::<T>::insert(who.clone(), key, (true, uri));

			// emit event
			Self::deposit_event(Event::NewRecordEntry { key: ekey, uri: cid });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn delete_data_entry(origin: OriginFor<T>, ekey: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut key: BoundedVec<_, T::MaxKeyLength> = Default::default();

			for j in ekey.clone() { key.try_push(j).map_err(|_| Error::<T>::KeyOverflow)? };

			// insert into storage
			match FragRecord::<T>::try_get(who.clone(), key.clone()) {
				Ok(record) => FragRecord::<T>::insert(who, key, (false, record.1)),
				Err(_) => {}
			}

			// emit event
			Self::deposit_event(Event::RecordEntryDeleted { key: ekey });

			Ok(())
		}
	}
}
