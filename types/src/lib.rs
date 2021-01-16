#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

mod protocol;
pub use protocol::*;

mod msg;
pub use msg::*;
pub use tokio_util::codec::{Decoder, Encoder};

pub type View = u64;
