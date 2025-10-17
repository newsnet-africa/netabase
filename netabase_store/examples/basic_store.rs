use crate::definitions::*;
use netabase_macros::netabase_definition_module;
use netabase_store::traits::model::NetabaseModel;

#[netabase_definition_module(ExampleDefs, ExampleDefKeys)]
pub mod definitions {
    use netabase_macros::NetabaseModel;

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug)]
    pub struct User {
        #[primary_key]
        pub name: String,
        pub age: u8,
        #[secondary_key]
        pub email: String,
    }
}

fn main() {
    let mut db = netabase_store::databases::sled_store::SledStore::<ExampleDefs>::temp()
        .expect("The store failed to open");
    let user_tree = db.open_tree::<User>();
    let user = User {
        name: "It's You!".to_string(),
        age: 24,
        email: "some@email.com".to_string(),
    };

    let put_result = user_tree.put(user.clone());

    let get_result = user_tree.get(user.primary_key());
    let get_secondary_result =
        user_tree.get_by_secondary_key(user.secondary_keys().first().unwrap().clone());

    println!("Get Result: {get_result:?}");
    println!("Get Secondary Result: {get_secondary_result:?}");

    assert!(put_result.is_ok());
    assert!(get_result.is_ok());
}
