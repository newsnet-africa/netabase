use netabase::NetabaseSchema;

#[derive(NetabaseSchema, Clone, bincode::Encode, bincode::Decode)]
struct SimpleKey {
    #[key]
    id: u64,
    name: String,
}

#[derive(NetabaseSchema, Clone, bincode::Encode, bincode::Decode)]
struct CompositeKey {
    #[key]
    user_id: u64,
    #[key]
    session_id: String,
    data: String,
}

#[derive(NetabaseSchema, Clone, bincode::Encode, bincode::Decode)]
struct TripleKey {
    #[key]
    part1: u32,
    #[key]
    part2: String,
    #[key]
    part3: u16,
    content: Vec<u8>,
}

fn main() {
    // Test simple key
    let simple = SimpleKey {
        id: 123,
        name: "test".to_string(),
    };
    let simple_key = simple.key();
    println!("Simple key: {:?}", simple_key);

    // Test composite key
    let composite = CompositeKey {
        user_id: 456,
        session_id: "abc123".to_string(),
        data: "some data".to_string(),
    };
    let composite_key = composite.key();
    println!("Composite key: {:?}", composite_key);

    // Test triple key
    let triple = TripleKey {
        part1: 789,
        part2: "xyz".to_string(),
        part3: 42,
        content: vec![1, 2, 3],
    };
    let triple_key = triple.key();
    println!("Triple key: {:?}", triple_key);

    println!("All tests completed successfully!");
}
