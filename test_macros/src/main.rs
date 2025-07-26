use netabase_macros::schemas;

trait SomeTrait {}

trait AnotherTrait<T: Clone> {}

#[schemas]
pub mod blah {
    use crate::AnotherTrait;
    use crate::NetabaseSchema;
    use crate::SomeTrait;
    use netabase_macros::NetabaseSchema;

    #[derive(NetabaseSchema)]
    pub struct NoDerive<T>(T);

    #[derive(NetabaseSchema)]
    pub struct New<T, B>
    where
        T: Clone + Default,
        B: Default,
    {
        s: T,
        a: B,
    }

    pub mod another {

        #[derive(Clone)]
        pub struct Child;
    }
}

fn main() {
    println!("FUCK")
}
