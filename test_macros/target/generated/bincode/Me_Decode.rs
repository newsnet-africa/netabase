impl < T : Encode + Decode, __Context > :: bincode :: Decode < __Context > for
Me < T > where T : :: bincode :: Decode < __Context >
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            first : :: bincode :: Decode :: decode(decoder) ?, second : ::
            bincode :: Decode :: decode(decoder) ?, blah : :: bincode ::
            Decode :: decode(decoder) ?,
        })
    }
} impl < '__de, T : Encode + Decode, __Context > :: bincode :: BorrowDecode <
'__de, __Context > for Me < T > where T : :: bincode :: de :: BorrowDecode <
'__de, __Context >
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            first : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, second : :: bincode :: BorrowDecode ::<
            '_, __Context >:: borrow_decode(decoder) ?, blah : :: bincode ::
            BorrowDecode ::< '_, __Context >:: borrow_decode(decoder) ?,
        })
    }
}