use netabase_macros::schema;

#[schema]
mod t {
    mod f {
        #[derive(Clone)]
        struct User {
            #[key]
            id: u128,
            name: String,
        }
    }

    #[derive(Clone)]
    #[key = |i: User2| i ]
    struct User2 {
        id: u128,
        name: String,
    }
}

fn main() {}
