impl :: bincode :: Encode for MyRegistryKey
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError > { match self { _ => core :: unreachable! () } }
}