use netabase_macros::schema;

#[schema]
mod t {

    #[derive(Clone)]
    struct User {
        #[key]
        id: u128,
        name: String,
        another: String
    }
}

fn main() {}
