

#[macro_use]
extern crate diesel;


mod codec;
pub use self::codec::{Jt808Codec,Jt808CodecError};

mod models;
pub use self::models::{Jt808,Position,Register,Msg8100,RegisterResult,Msg8203,GeneralResponse,business,
    Authentication,
    schema
};