impl < __Context > :: bincode :: Decode < __Context > for MyRegistryKey
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Err(::bincode::error::DecodeError::EmptyEnum
        { type_name: core::any::type_name::<Self>() })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for MyRegistryKey
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Err(::bincode::error::DecodeError::EmptyEnum
        { type_name: core::any::type_name::<Self>() })
    }
}