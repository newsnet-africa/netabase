impl :: bincode :: Encode for You
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        match self
        {
            Self ::First(field_0, field_1)
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (0u32), encoder) ?
                ; :: bincode :: Encode :: encode(field_0, encoder) ?; ::
                bincode :: Encode :: encode(field_1, encoder) ?; core ::
                result :: Result :: Ok(())
            }, Self ::Second(field_0)
            =>{
                < u32 as :: bincode :: Encode >:: encode(& (1u32), encoder) ?
                ; :: bincode :: Encode :: encode(field_0, encoder) ?; core ::
                result :: Result :: Ok(())
            },
        }
    }
}