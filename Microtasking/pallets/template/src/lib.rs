#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};

use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
	traits::{
		Currency, ExistenceRequirement, Get, LockIdentifier, LockableCurrency, ReservableCurrency,
		WithdrawReasons,
	},
};
use frame_system::ensure_signed;
use pallet_assets;
use pallet_balances;
use pallet_staking;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const LOCKSECRET: LockIdentifier = *b"mylockab";

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq)]
pub struct ActiveTask<AccountId, Balance> {
	task_id: u128,
	client: AccountId,
	worker_id: Option<AccountId>,
	task_deadline: u64,
	task_description: Vec<u8>,
	cost: Balance,
	is_bidded: bool,
}

#[derive(Encode, Decode, Default, Debug, PartialEq, Clone, Eq)]
pub struct DeadTask<AccountId, Balance> {
	task_id: u128,
	client: AccountId,
	worker_id: Option<AccountId>,
	task_deadline: u64,
	task_description: Vec<u8>,
	cost: Balance,
	is_bidded: bool,
}

pub trait Config: frame_system::Config {

	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: LockableCurrency<Self::AccountId>;
}


decl_storage! {

	trait Store for Module<T: Config> as TemplateModule {


            //store taskid -> active_task_list.
			IDtoTask get(fn active_task):
            map hasher(blake2_128_concat) u128 => ActiveTask<T::AccountId, BalanceOf<T>>;
               //store taskid -> dead_task_list.
            IDtoTaskD get(fn dead_task):
            map hasher(blake2_128_concat) u128 => DeadTask<T::AccountId, BalanceOf<T>>;
            //stores count for key in map.
            ActiveTaskCount get(fn active_task_count): u128 = 0;
            //stores taskid -> staker/bidder list.
			IDtoStaker get(fn staker_list):
			map hasher(blake2_128_concat) u128 => Vec<T::AccountId>;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
		Balance = BalanceOf<T>,
	{
        TaskCreated(AccountId, u128, u64, Vec<u8>, Balance),
        TaskCompleted(AccountId, u128),
		StakerAdded(AccountId,u128,AccountId),
        AmountTransfered(AccountId,AccountId,Balance),
        
	}
);


decl_error! {
	pub enum Error for Module<T: Config> {


		NoneValue,
		StorageOverflow,
		OriginNotSigned,
		NotEnoughBalance,
		TaskDoesNotExist,
        AlreadyMember,
    
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

        fn deposit_event() = default;
        #[weight = 10_000]
		pub fn publish_task(origin, task_dur: u64, task_des: Vec<u8>, task_cost: BalanceOf<T>) -> dispatch::DispatchResult {
		 let sender = ensure_signed(origin)?;
		 let current_count = Self::active_task_count();

	

		 let task_struct= ActiveTask {
			  task_id: current_count.clone(),
			  client:sender.clone(),
			  worker_id: None,
			  task_deadline: task_dur.clone(),
			  task_description: task_des.clone(),
			  cost:task_cost.clone(),
			  is_bidded: Default::default(),
		  };
		  IDtoTask::<T>::insert(current_count.clone(),task_struct);
		  Self::deposit_event(RawEvent::TaskCreated(sender, current_count.clone(), task_dur.clone(), task_des.clone(), task_cost.clone()));
		  ActiveTaskCount::put(current_count + 1);
		  Ok(())
        }




        #[weight = 10_000]
		pub fn bid_for_task(origin, task_id: u128) {
			let bidder = ensure_signed(origin)?;
			ensure!(Self::task_exist(task_id.clone()), Error::<T>::TaskDoesNotExist);
            let mut task_struct = Self::get_task_A(task_id.clone());
            let publisher=task_struct.client.clone();
			task_struct.worker_id = Some(bidder.clone());
			task_struct.is_bidded = true;
			IDtoTask::<T>::insert(&task_id,task_struct);
			Self::deposit_event(RawEvent::StakerAdded(publisher.clone(), task_id.clone(),bidder));
        }



      
             #[weight = 10_000]
		pub fn task_completed(origin, task_id: u128) {
            let publisher=ensure_signed(origin)?;
            ensure!(Self::task_exist(task_id.clone()), Error::<T>::TaskDoesNotExist);
        
            let mut active_task_struct=Self::get_task_A(task_id.clone());
            //Need to find short cut for copying from-to struct.
                let dead_task_struct= DeadTask {
			  task_id: active_task_struct.task_id,
			  client:active_task_struct.client,
			  worker_id: active_task_struct.worker_id,
			  task_deadline:active_task_struct.task_deadline,
			  task_description:active_task_struct.task_description,
			  cost:active_task_struct.cost,
			  is_bidded:active_task_struct.is_bidded,
          };
          
           IDtoTaskD::<T>::insert(&task_id,dead_task_struct);
           //cannot emit bidder,coz type is option.
          Self::deposit_event(RawEvent::TaskCompleted(publisher.clone(), task_id.clone()));
            
        }
     


	}
}

impl<T: Config> Module<T> {
	// Helper functions

    //Active task
	pub fn get_task_A(task_id: u128) -> ActiveTask<T::AccountId, BalanceOf<T>> {
		IDtoTask::<T>::get(&task_id)
    }
    
    pub fn task_exist(task_id: u128)->bool{
         IDtoTask::<T>::contains_key(&task_id)
    }

    //Dead task
    pub fn get_task_D(task_id: u128) -> DeadTask<T::AccountId, BalanceOf<T>> {
		IDtoTaskD::<T>::get(&task_id)
    }

}