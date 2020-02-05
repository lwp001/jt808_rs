use bytes::Buf; //Bytes
use lwp_util::byte_help::utf8_string;

use super::{Jt808,Jt808Trans};

/// 0102 鉴权信息
#[derive(Debug)]
pub struct Authentication {
    /// 设备SIM
    pub sim: String,
    /// 鉴权码
    pub code: i64,
    /// 通讯设备 IMEI
    pub imei:Option<String>,
    /// 软件版本
    pub ver:Option<String>,
    /// 解析结果 0成功 其他错误
    pub is_err: i32,
}

impl Jt808Trans for Authentication  {
    fn trans(jt808:&mut Jt808,_vin:&str)->Self {
        let sim=jt808.get_sim();
        let buf= &mut jt808.body;
        if jt808.ver.is_none() {
            Authentication {
                sim,
                code: buf.get_i64(),
                imei: None,
                ver: None,
                is_err: 0,
            }
        } else {
            //鉴权码长度
            if buf.get_u8() !=8 {
               return Authentication {
                    sim,
                    code:0,
                    imei:None,
                    ver: None,
                    is_err: 0,
                };
            }
             Authentication {
                sim,
                code: buf.get_i64(),
                imei: Some(utf8_string(&buf.split_to(15))),
                ver: Some(utf8_string(&buf.split_to(20))),
                is_err: 0,
            }
        }
    }
}
