use super::Codec;
use crate::{Code, Status};
use bytes::{BufMut, BytesMut};
use prost::Message;
use std::marker::PhantomData;
use tokio_codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct ProstCodec<T, U> {
    _pd: PhantomData<(T, U)>,
}

impl<T, U> ProstCodec<T, U> {
    pub fn new() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T, U> Codec for ProstCodec<T, U>
where
    T: Message,
    U: Message + Default,
{
    type Encode = T;
    type Decode = U;

    type Encoder = ProstEncoder<T>;
    type Decoder = ProstDecoder<U>;

    const CONTENT_TYPE: &'static str = "application/groc+proto";

    fn encoder(&mut self) -> Self::Encoder {
        ProstEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        ProstDecoder(PhantomData)
    }
}

pub struct ProstEncoder<T>(PhantomData<T>);

impl<T: Message> Encoder for ProstEncoder<T> {
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let len = item.encoded_len();

        if buf.remaining_mut() < len {
            buf.reserve(len);
        }

        item.encode(buf)
            .map_err(|_| unreachable!("Message only errors if not enough space"))
    }
}

pub struct ProstDecoder<U>(PhantomData<U>);

impl<U: Message + Default> Decoder for ProstDecoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Message::decode(buf.take())
            .map(Option::Some)
            .map_err(from_decode_error)
    }
}

fn from_decode_error(error: prost::DecodeError) -> crate::Status {
    // Map Protobuf parse errors to an INTERNAL status code, as per
    // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
    Status::new(Code::Internal, error.to_string())
}