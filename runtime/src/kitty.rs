use frame_support::{decl_storage, decl_module,decl_event,StorageValue,dispatch,ensure};
use frame_support::traits::{Currency,Randomness,ExistenceRequirement::AllowDeath,ReservableCurrency};
use frame_system::{self as system, ensure_signed};
use frame_support::codec::{Encode, Decode};
use sp_core::H256;
use sp_std::cmp;
//use sp_std::vec::Vec;
use sp_runtime::traits::{Hash,Zero};
//use sp_runtime::traits::Zero;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
pub trait Trait:system::Trait {
   
    type RandomnessSource: Randomness<H256>;
    type Event:From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

}

//type ReportIdOf<T> = <T as frame_system::Trait>::Hash;
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MyStruct<Hash, Balance>{
    gen: u64,
    id: Hash,
    dna: Hash,
    price: Balance,
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
        <T as frame_system::Trait>::Hash,
        Balance = BalanceOf<T>
    {
        Created(AccountId, Hash),
        PriceSet(AccountId, Hash, Balance),
        Transferred(AccountId, AccountId, Hash),
        Bought(AccountId, AccountId, Hash, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // Declare storage and getter functions here
        // MyItem get(fn my_item):map hasher(blake2_128_concat)T::AccountId=>MyStruct<T::Balance, T::Hash>;
        // MyU32 get(fn my_u32):u32;
        //MyBool get(fn my_bool):bool; 
        //SomeValue get(fn some_value):map hasher(blake2_128_concat) u32=>u32;
        // MyValue get(fn my_value): map hasher(blake2_128_concat)T::AccountId => u32;
        //猫的数组,[编号,猫ID]
        AllKittiesArray get(fn kitty_by_index):map hasher(blake2_128_concat)u64=> T::Hash;
        //猫的数量,一般是猫的最后一个编号+1
        AllKittiesCount get(fn all_kitties_count):u64;
        //猫的ID对应猫的编号
        AllKittiesIndex :map hasher(blake2_128_concat)T::Hash =>u64;
        //账户对应猫的ID
        //OwnedKitty get(fn own_kitty):map hasher(blake2_128_concat)T::AccountId => T::Hash;
        
        //猫的ID对应账户
        KittyOwner get(fn kitty_owner):map hasher(blake2_128_concat) T::Hash=> Option<T::AccountId>;
        //猫的信息
        Kitties get(fn kitty):map hasher(blake2_128_concat) T::Hash=> MyStruct<T::Hash, BalanceOf<T>>;
        //账户拥有多只猫的储存结构 [账户,猫的编号]=>猫的ID
        OwnedKittiesArray get(fn kitty_of_owner_by_index): map hasher(blake2_128_concat)(T::AccountId, u64) => T::Hash;
        //账户拥有的猫的数量
        OwnedKittiesCount get(fn owned_kitty_count): map hasher(blake2_128_concat)T::AccountId => u64;
        //猫的ID对应猫的编号
        OwnedKittiesIndex: map hasher(blake2_128_concat)T::Hash => u64;
        Nonce get(fn nonce):u64;

  }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Declare public functions here
       /* #[weight = 0] 
        fn my_funtion(origin, input_bool:bool) -> dispatch::DispatchResult {
            let _sender = ensure_signed(origin)?;

            MyBool::put(input_bool);

            Ok(())
        }
        #[weight = 0] 
        fn set_value(origin, value:u32) -> dispatch::DispatchResult {
            let _sender = ensure_signed(origin)?;
            MyU32::put(value);
            Ok(())
        }
        #[weight = 0]
        fn set_account_value(origin,num:u32)-> dispatch::DispatchResult{
            let sender = ensure_signed(origin)?;
            <MyValue<T>>::insert(sender,num);
            Ok(())
        }*/
        fn deposit_event() = default;
        #[weight = 0]
        fn create_struct(origin) ->dispatch::DispatchResult{
            //let sender = ensure_signed(origin)?;
            //let new_struct = MyStruct::default();
            //Ok(())
            //<OwnedKitty<T>>::insert(sender,new_struct);
            let sender = ensure_signed(origin)?;

            let nonce = Nonce::get();
            let random_hash = ( T::RandomnessSource::random_seed(), &sender, nonce)
                .using_encoded(T::Hashing::hash);
            
            let new_kitty = MyStruct {
                gen: 0,
                id: random_hash,
                dna: random_hash,
                price: Default::default(),
            };

            Self::mint(sender, random_hash, new_kitty)?;
            Nonce::mutate(|n| *n += 1);

            Ok(())
        }
        #[weight = 0]  
        fn set_price(origin, kitty_id: T::Hash, new_price: BalanceOf<T>) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");

            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this cat");

            let mut kitty = Self::kitty(kitty_id);
            kitty.price = new_price;

            <Kitties<T>>::insert(kitty_id, kitty);

            Self::deposit_event(RawEvent::PriceSet(sender, kitty_id, new_price));

            Ok(())
        }
        #[weight = 0]  
        fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> dispatch::DispatchResult  {
            let sender = ensure_signed(origin)?;
            //检查猫ID是否有拥有者，如果有则返回ACCOUNTID,无则报错
            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            //检查拥有者和发起者是否是同一个账户，如果不是就报错
            ensure!(owner == sender, "You do not own this kitty");

            Self::transfer_from(sender, to, kitty_id)?;

            Ok(())
        }
        #[weight = 0]
        fn buy_kitty(origin, kitty_id: T::Hash, max_price: BalanceOf<T>) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;
            //检查猫ID是否有对应的猫存在
            ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");
            //检查猫ID是否有主人
            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            //如果买猫的人和他的主人相同，就报错
            ensure!(owner != sender, "You can't buy your own cat");
            //通过猫ID获取猫本身
            let mut kitty = Self::kitty(kitty_id);
            //获取价格
            let kitty_price = kitty.price;
            //如果价格是0就报错,不是0才能卖，是0表示不可卖
            ensure!(!kitty_price.is_zero(), "The cat you want to buy is not for sale");
            //价格大于你的余额时报错
            ensure!(kitty_price <= max_price, "The cat you want to buy costs more than your max price");
            //转账
            T::Currency::transfer(&sender, &owner, kitty_price,AllowDeath)?;
            //猫的拥有权发生转移
            Self::transfer_from(owner.clone(), sender.clone(), kitty_id)
                .expect("`owner` is shown to own the kitty; \
                `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
                `all_kitty_count` shares the same type as `owned_kitty_count` \
                and minting ensure there won't ever be more than `max()` kitties, \
                which means transfer cannot cause an overflow; \
                qed");
            //卖完后价格归0改为不可卖
            kitty.price = Default::default();
            //更新猫的价格
            <Kitties<T>>::insert(kitty_id, kitty);
            //触发事件
            Self::deposit_event(RawEvent::Bought(sender, owner, kitty_id, kitty_price));

            Ok(())
        }
        #[weight = 0]
        fn breed_kitty(origin, kitty_id_1: T::Hash, kitty_id_2: T::Hash) -> dispatch::DispatchResult{
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::contains_key(kitty_id_1), "This cat 1 does not exist");
            ensure!(<Kitties<T>>::contains_key(kitty_id_2), "This cat 2 does not exist");

            let nonce = Nonce::get();
            let random_hash = (T::RandomnessSource::random_seed(), &sender, nonce)
                .using_encoded(T::Hashing::hash);

            let kitty_1 = Self::kitty(kitty_id_1);
            let kitty_2 = Self::kitty(kitty_id_2);

            let mut final_dna = kitty_1.dna;
            for (i, (dna_2_element, r)) in kitty_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
                if r % 2 == 0 {
                    final_dna.as_mut()[i] = *dna_2_element;
                }
            }

            let new_kitty = MyStruct {
                gen: cmp::max(kitty_1.gen, kitty_2.gen) + 1,
                id: random_hash,
                dna: final_dna,
                price: Default::default(),
                
            };

            Self::mint(sender, random_hash, new_kitty)?;

            Nonce::mutate(|n| *n += 1);

            Ok(())
        }

    }
}


impl<T: Trait> Module<T> {
    fn mint(to: T::AccountId, kitty_id: T::Hash, new_kitty: MyStruct<T::Hash, BalanceOf<T>>) -> dispatch::DispatchResult {
        ensure!(!<KittyOwner<T>>::contains_key(&kitty_id), "Kitty already exists");

        let owned_kitty_count = Self::owned_kitty_count(&to);

        let new_owned_kitty_count = owned_kitty_count.checked_add(1)
        .ok_or("Overflow adding a new kitty to account balance")?;

        let all_kitties_count = Self::all_kitties_count();

        let new_all_kitties_count = all_kitties_count.checked_add(1)
        .ok_or("overflow adding a new kitty to total supply")?;
  
        <Kitties<T>>::insert(kitty_id, new_kitty);
        <KittyOwner<T>>::insert(kitty_id, &to);
        //<OwnedKitty<T>>::insert(&to, kitty_id);

        <AllKittiesArray<T>>::insert(all_kitties_count, kitty_id);
        AllKittiesCount::put(new_all_kitties_count);
        <AllKittiesIndex<T>>::insert(kitty_id, all_kitties_count);
        
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count);

        Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }

    fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> dispatch::DispatchResult {
        let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;

        ensure!(owner == from, "'from' account does not own this kitty");
        //发起者的猫总数
        let owned_kitty_count_from = Self::owned_kitty_count(&from);
        //送达者的猫总数
        let owned_kitty_count_to = Self::owned_kitty_count(&to);
        //查看送达者猫总数加1是否会溢出
        let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1)
            .ok_or("Transfer causes overflow of 'to' kitty balance")?;
        //查看发起者猫总数-1是否会不够,同时也是最后一个猫的编号
        let new_owned_kitty_count_from = owned_kitty_count_from.checked_sub(1)
            .ok_or("Transfer causes underflow of 'from' kitty balance")?;
        //通过猫ID得到猫的对应编号      
        let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);

        //判断  得到的猫编号不是最后一个
        if kitty_index != new_owned_kitty_count_from {
            //得到最后一个猫的哈希ID
            let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
            //把最后一个猫放到送出去的猫的位置
            <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
            //更新原来最后一个猫哈希ID对应的编号
            <OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
        }
        //
        //把猫的ID映射到送达者
        <KittyOwner<T>>::insert(&kitty_id, &to);
        //更新猫在送达者的新ID
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);
        //移除发起者的最后一个猫
        <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
        //把猫的ID，编号赋值给送达者
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);
        //更新发起者的猫总数
        <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
        //更新收到者的猫总数
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);
        
        Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));

        Ok(())
    }
}