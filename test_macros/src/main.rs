use netabase_macros::schemas;

#[schemas]
mod blah {
    #[derive(Clone)]
    struct NoDerive;

    #[derive(Clone)]
    struct New;

    mod another {

        #[derive(Clone)]
        struct Child;
    }
}

fn main() {
    println!("FUCK")
}
