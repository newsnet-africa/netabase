impl < __Context > :: bincode :: Decode < __Context > for UserRecord
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            user_id : :: bincode :: Decode :: decode(decoder) ?, username : ::
            bincode :: Decode :: decode(decoder) ?, email : :: bincode ::
            Decode :: decode(decoder) ?, created_at : :: bincode :: Decode ::
            decode(decoder) ?,
        })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for UserRecord
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            user_id : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, username : :: bincode :: BorrowDecode
            ::< '_, __Context >:: borrow_decode(decoder) ?, email : :: bincode
            :: BorrowDecode ::< '_, __Context >:: borrow_decode(decoder) ?,
            created_at : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?,
        })
    }
}