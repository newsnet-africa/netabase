mod t {

    use netabase_macros::schema;

    #[schema]
    struct User {
        id: u128,
        name: String,
    }
}

fn main() {}
