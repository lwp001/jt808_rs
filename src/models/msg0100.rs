use bytes::{Bytes,Buf,BytesMut,BufMut};
use lwp_util::byte_help::{utf8_string,gbk_string};

use super::{Jt808,Jt808Trans,Jt808BodySerialize};


/// 0100 注册信息
#[derive(Debug)]
pub struct Register {
    /// bcd 6/10 bytes
    pub sim:String,
    /// province
    pub province:u16,
    /// city
    pub city:u16,
    /// 5 bytes
    pub producer:String,
    /// 20 bytes
    pub terminal_model:String,
    /// 7 bytes
    pub terminal_id:String,
    /// platenumber color
    pub color:u8,
    /// vin
    pub vin:String,
    /// register result
    pub result: i16,
}

impl Jt808Trans for Register  {
    fn trans(jt808:&mut Jt808,_vin:&str)->Self {
        let sim=jt808.get_sim();
        let buf= &mut jt808.body;
        let province = buf.get_u16();
        let city = buf.get_u16();
        if jt808.ver.is_none() {
            return   Register {
                        sim,
                        province,
                        city,
                        producer: utf8_string(&buf.split_to(5)),
                        terminal_model: utf8_string(&buf.split_to(20)),
                        terminal_id: utf8_string(&buf.split_to(7)),
                        color: buf.get_u8(),
                        vin: gbk_string(&buf[..]),
                        result: 11,
            }
        }
        Register {
            sim,
            province,
            city,
            producer: utf8_string(&buf.split_to(11)),
            terminal_model: utf8_string(&buf.split_to(30)),
            terminal_id: utf8_string(&buf.split_to(30)),
            color: buf.get_u8(),
            vin: gbk_string(&buf[..]),
            result: 11,
        }
    }
}

#[derive(Debug)]
pub struct Msg8100 {
    pub serial: u16,
    /// 0：成功；1：车辆已被注册；2：数据库中无该车辆；3：终端已被注册；4：数据库中无该终端
    pub result: RegisterResult,
    pub auth_code: Option<i64>,
}
#[derive(Debug,PartialEq)]
pub enum RegisterResult {
  Success = 0,
  VehicelRegistered = 1,
  VehicleWithout = 2,
  TerminalRegistered = 3,
  TerminalWithout = 4,
  RegisterError = 5,
}

impl Jt808BodySerialize for Msg8100  {
    fn serialize(self) -> Bytes {
        let mut buf= BytesMut::with_capacity(9);
        buf.put_u16(self.serial);
        buf.put_u8(self.result as u8);
       if buf[2] == 0u8 {       
            buf.put_i64(self.auth_code.unwrap());
        }
        buf.freeze() 
    }
}

impl Msg8100 {
    pub fn new(serial: u16,result: RegisterResult, auth_code: Option<i64>) -> Self{
        Msg8100 {
            serial,
            result,
            auth_code,
        }
    }
}