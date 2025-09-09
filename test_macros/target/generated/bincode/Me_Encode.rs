impl < T : Encode + Decode > :: bincode :: Encode for Me < T > where T : ::
bincode :: Encode
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.first, encoder) ?; :: bincode ::
        Encode :: encode(&self.second, encoder) ?; :: bincode :: Encode ::
        encode(&self.blah, encoder) ?; core :: result :: Result :: Ok(())
    }
}