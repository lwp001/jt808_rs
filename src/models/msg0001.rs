use bytes::{Bytes,Buf,BytesMut,BufMut};
use super::{Jt808,Jt808Trans,Jt808BodySerialize};
/// 0001 or 8001 通用应答
#[derive(Debug)]
pub struct GeneralResponse {
    /// 对应需要应答的流水号 
    pub serial:u16,
    /// 需要应答的命令字
    pub cmd:u16,
    /// 结果 0:成功/确认 1:失败 2:消息有误 3:不支持 4:报警处理确认
    pub result:u8,
}

impl Jt808Trans for GeneralResponse  {
    fn trans(jt808:&mut Jt808,_vin:&str)->Self {
        let buf= &mut jt808.body;
        GeneralResponse {
            serial:  buf.get_u16(),
            cmd:  buf.get_u16(),
            result: buf.get_u8(),
        }
    }
}

impl Jt808BodySerialize for GeneralResponse  {
    fn serialize(self) -> Bytes {
            let mut buf= BytesMut::with_capacity(5);
            buf.put_u16(self.serial);
            buf.put_u16(self.cmd);
            buf.put_u8(self.result);
            buf.freeze()
    }
}