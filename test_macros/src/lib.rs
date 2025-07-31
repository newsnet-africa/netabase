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
<<<<<<< HEAD
    struct User {
        #[key]
=======
    #[key = |i: User2| i ]
    struct User2 {
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
        id: u128,
        name: String,
        another: String
    }
}

fn main() {}
