use crate::nodes::discovery::schemas::TestItem;
use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase::netabase_trait::NetabaseSchema;
use netabase_macros::NetabaseSchema;
use netabase_macros::NetabaseSchemaKey;
use netabase_macros::schema_module;

#[schema_module(MyRegistery)]
pub mod schemas {
    use bincode::{Decode, Encode};
    use netabase::netabase_trait::NetabaseSchema;
    use netabase_macros::NetabaseSchema;
    use netabase_macros::NetabaseSchemaKey;
    use netabase_macros::schema_module;

    #[derive(NetabaseSchema, Encode, Decode, Debug, Clone)]
    pub struct TestItem {
        #[key]
        pub some_key: String,
        pub other_value: String,
    }
}
#[tokio::test]
async fn scratch1() {
    let test_item = TestItem {
        some_key: "Some".to_string(),
        other_value: "Other".to_string(),
    };
    let mut net1 = Netabase::<MyRegisteryKey, MyRegistery>::new_test(1);
    let mut list = net1.swarm_event_listener.resubscribe();
    println!("{:?}", list.recv().await);
    net1.close().await;
}
#[tokio::test]
async fn scratch2() {
    let test_item = TestItem {
        some_key: "Some".to_string(),
        other_value: "Other".to_string(),
    };
    let mut net2 = Netabase::<MyRegisteryKey, MyRegistery>::new_test(2);
    let mut list = net2.swarm_event_listener.resubscribe();
    println!("{:?}", list.recv().await);
    net2.close().await;
}
