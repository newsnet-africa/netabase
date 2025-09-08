impl :: bincode :: Encode for Me
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.first, encoder) ?; :: bincode ::
        Encode :: encode(&self.second, encoder) ?; core :: result :: Result ::
        Ok(())
    }
}