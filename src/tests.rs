use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn  create_kitty_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(&1), 100);
        assert_eq!(Balances::reserved_balance(&1), 0);
        assert_ok!(KittiesModule::create(Origin::signed(1), 10 ));
        assert_eq!(Balances::free_balance(&1), 90);
        assert_eq!(Balances::reserved_balance(&1), 10);
        assert_eq!(
            Owner::<Test>::get(0),
            Option::Some(1)
        );

    });
}


#[test]
fn  transfer_kitty_works() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
        assert_ok!(KittiesModule::transfer(Origin::signed(1), 2, 0 ));
        assert_eq!(
            Owner::<Test>::get(0),
            Option::Some(2)
        );

    });
}

#[test]
fn  transfer_kitty_faild_when_invalidKittyIndex() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 2, 1 ),
            Error::<Test>::InvalidKittyIndex
        );

    });
}



#[test]
fn  transfer_kitty_faild_when_notowner() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
		assert_noop!(
			KittiesModule::transfer(Origin::signed(2), 3, 0 ),
            Error::<Test>::NotOwner
        );

    });
}

#[test]
fn  breed_kitty_work() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
        KittiesModule::create(Origin::signed(2), 10 );
        assert_ok!(KittiesModule::breed(Origin::signed(1), 0, 1 ));
        assert_eq!(
            Owner::<Test>::get(2),
            Option::Some(1)
        );


    });
}


#[test]
fn  breed_kitty_failed_when_sameParentIndex() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
        KittiesModule::create(Origin::signed(2), 10 );

        assert_noop!(
			KittiesModule::breed(Origin::signed(1), 1, 1 ),
            Error::<Test>::SameParentIndex
        );

    });
}



#[test]
fn  offer_kitty_works() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
        assert_ok!(KittiesModule::offer(Origin::signed(1), 20, 0 ));
        assert_eq!(
            Offers::<Test>::get(0),
            Some(20)
        );



    });
}




#[test]
fn  offer_kitty_failed_when_notOwner() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );

        assert_noop!(
            KittiesModule::offer(Origin::signed(2), 20, 0 ),
            Error::<Test>::NotOwner
        );

    });
}


#[test]
fn  buy_kitty_works() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
        assert_ok!(KittiesModule::offer(Origin::signed(1), 20, 0 ));
        assert_eq!(Balances::free_balance(&2), 100);
        assert_ok!(KittiesModule::buy(Origin::signed(2), 0 ));
        assert_eq!(Balances::free_balance(&2), 80);
        assert_eq!(
            Owner::<Test>::get(0),
            Option::Some(2)
        );

        assert_eq!(
            Offers::<Test>::get(0),
            None
        );


    });
}

#[test]
fn  buy_kitty_failed_notOffer() {
    new_test_ext().execute_with(|| {
        KittiesModule::create(Origin::signed(1), 10 );
        assert_noop!(
            KittiesModule::buy(Origin::signed(2), 0 ),
            Error::<Test>::NotOffer
        );


    });
}



