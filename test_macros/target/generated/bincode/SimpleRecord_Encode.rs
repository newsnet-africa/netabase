impl :: bincode :: Encode for SimpleRecord
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.record_id, encoder) ?; :: bincode
        :: Encode :: encode(&self.name, encoder) ?; :: bincode :: Encode ::
        encode(&self.description, encoder) ?; core :: result :: Result ::
        Ok(())
    }
}