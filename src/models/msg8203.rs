

use bytes::{Bytes,BytesMut,BufMut};
/// 8203 报警确认
#[derive(Debug)]
pub struct Msg8203 {
    /// response serial 0200
    pub res_serial :i16,
    /// 报警信息
    pub alarm: i32,
}

impl Msg8203 {
    /// 8203 to bytes
    pub fn to_bytes(self) -> Bytes {
        let mut body= BytesMut::with_capacity(6);
        body.put_i16(self.res_serial);
        body.put_i32(self.alarm);
        body.freeze()
    }
}