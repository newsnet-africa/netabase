impl :: bincode :: Encode for UserRecord
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.user_id, encoder) ?; :: bincode
        :: Encode :: encode(&self.username, encoder) ?; :: bincode :: Encode
        :: encode(&self.email, encoder) ?; :: bincode :: Encode ::
        encode(&self.created_at, encoder) ?; core :: result :: Result ::
        Ok(())
    }
}