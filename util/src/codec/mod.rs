use tokio_util::codec::LengthDelimitedCodec;

pub mod block;
pub mod tx;

pub mod proto;
pub mod synchs;

#[derive(Debug)]
pub struct EnCodec(pub LengthDelimitedCodec);

impl EnCodec {
    pub fn new() -> Self {
        EnCodec(LengthDelimitedCodec::new())
    }
}

impl std::clone::Clone for EnCodec {
    fn clone(&self) -> Self {
        EnCodec::new()
    }
}
