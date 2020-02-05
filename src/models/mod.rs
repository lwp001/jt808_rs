
use bytes::{Bytes,Buf,BytesMut,BufMut};
use lwp_util::byte_help::bcd_sim;

mod msg0001;
pub use self::msg0001::GeneralResponse;
mod msg0200;
pub use self::msg0200::Position;
mod msg0100;
pub use self::msg0100::{Register,Msg8100,RegisterResult};
mod msg0102;
pub use self::msg0102::Authentication;

mod msg8203;
pub use self::msg8203::Msg8203;
pub mod business;

pub mod schema;

/// Jt808 解析后的结构体 包含2019、2013版本
#[derive(Debug,Clone)]
pub struct Jt808 {
    /// 命令字
    pub cmd: u16,
    /// 2019 keep 15 ver_flag=1:14
    pub keep2bit: u16,
    /// 分包标志
    pub package_flag_1bit: u16,
    /// 加密标志
    pub encrypt3bit: u16,
    /// body length
    pub body_length: u16,
    /// version
    pub ver: Option<u8>,
    /// bcd sim 2019=10bytes 2013=6bytes
    pub bcd_sim: Vec<u8>,
    /// serial
    pub serial: u16,
    /// 总分包数
    pub package_total: Option<u16>,
    /// 当前上传分包索引
    pub package_index: Option<u16>,
    /// body
    pub body: Bytes,
}


impl From<Bytes> for Jt808 {
    fn from(buf: Bytes) -> Self {
        //帧长度验证 帧头的长度字段是否正确
        //let frame_len=buf.len() as u16;
        let mut buf= buf;//.bytes();//Bytes::from(&buf[..]).into_buf();//buf.bytes().into_buf();
        //跳过帧标识 0x7E
        buf.advance(1);
        let cmd=buf.get_u16();
        let length=buf.get_u16();
        let keep2bit= &length >> 14 & 0b11_u16;
        let package_flag_1bit = &length >> 13 & 0b1_u16;
        let encrypt3bit = &length >> 12 & 0b111_u16;
        let body_length = length & 0x3ff;
        let mut bcd_sim: Vec<u8>=Vec::new();
        let ver: Option<u8>;//=None;
        if  buf.len()- (body_length as usize) > 14
        {
            ver=Some(buf.get_u8());
            // buf.split_to(10);
            bcd_sim.put(buf.split_to(10));
        } else {
            ver= None;
            bcd_sim.put(buf.split_to(6));
        }
        // let mut sim=[0u8;6];
        // buf.copy_to_slice(&mut sim);
        let serial = buf.get_u16();
        //let real_body_length= (package_flag_1bit as u16) * 4 + body_length;
        //判断数据体长度是否正确
        //if real_body_length > frame_len - 15_u16 {
        //    return Error("");
        //}
        let (package_total,package_index) = if package_flag_1bit==1
        {
            (Some(buf.get_u16()),Some(buf.get_u16()))
            //判断是否需要重新传送
        } else {
            (None,None)
        };
        let body= if body_length > 0 {
            //去掉校验字节 和 0x7E
            (&buf[..(body_length as usize)]).to_bytes()
            
        }else {
            Bytes::new()
        };

        Jt808{
            cmd,
            keep2bit,
            package_flag_1bit,
            encrypt3bit,
            body_length,
            ver,
            bcd_sim,
            serial,
            package_total,
            package_index,
            body,
        }
    }
}
impl From<Jt808> for Bytes  {
    fn from(jt808: Jt808) -> Self {
        let body_length=jt808.body_length.to_owned();
        let mut buf= BytesMut::with_capacity(body_length as usize + 18);
        buf.put_u16(jt808.cmd);
        let length_prop= jt808.keep2bit << 14 | jt808.package_flag_1bit << 13 | jt808.encrypt3bit << 12 | jt808.body_length;
        buf.put_u16(length_prop);
        if let Some(ver) =jt808.ver  {
            buf.put_u8(ver);
        }
        buf.put(&jt808.bcd_sim[..]);
        buf.put_u16(jt808.serial);
        if let Some(package_total)=jt808.package_total {
            buf.put_u16(package_total);
        }
        if let Some(package_index) = jt808.package_index   {
            buf.put_u16(package_index);
            //判断是否需要重新传送
        }
        // if let Some(body)= jt808.body {
        buf.put(jt808.body);
        // }
        // let temp=buf.to_bytes();
        // 可以适当放大容量 不用在重新扩充空间
        let mut xor=0_u8;
        for b in buf.iter() {
            xor ^=*b;
        }
        buf.put_u8(xor);
        buf.freeze()
    }
}

// pub trait Jt808Trans {
//     fn trans(buf: &mut Bytes,vin: &String,ver: &i16) -> Self;
// }

pub trait Jt808Trans {
    fn trans(jt808: &mut Jt808,vin: &str) -> Self;
}

pub trait Jt808BodySerialize {
    fn serialize(self) -> Bytes;
}


impl Jt808 {
    /// 获取SIM
    pub fn get_sim(&self) -> String {
        // let s=format!("{}", self.bcd_sim.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(""));
        // s
        bcd_sim(&self.bcd_sim)
    }
    /// 重建jt808
    pub fn renew<T: Jt808BodySerialize>(&self,cmd:u16,serial:u16,body:T) -> Self {
        let body= body.serialize();
        Jt808 {
            cmd,
            serial,
            body_length: body.len() as u16,
            body,
            keep2bit: self.keep2bit,
            package_flag_1bit: self.package_flag_1bit,
            encrypt3bit: self.encrypt3bit,
            ver: self.ver,
            bcd_sim: self.bcd_sim.clone(),
            package_total: self.package_total,
            package_index: self.package_index,
        }
    }
    /// 更新JT808
    pub fn change_no_subpacket<T: Jt808BodySerialize>(&mut self,cmd:u16,body:T) {
        let body= body.serialize();
        self.cmd=cmd;
        self.body_length =body.len() as u16;
        self.body=body;
        self.keep2bit=0;
        self.package_flag_1bit=0;
        self.encrypt3bit=0;
        self.package_total=None;
        self.package_index=None;
    }
    /// serial 通用应答
    pub fn common_response(&mut self,result:u8)  {
        let gen= GeneralResponse {
            serial: self.serial,
            cmd: self.cmd,
            result,
        };
        let body= gen.serialize();
        self.cmd=0x8001;
        self.body_length = 5;
        self.body=body;
        self.keep2bit=0;
        self.package_flag_1bit=0;
        self.encrypt3bit=0;
        self.package_total=None;
        self.package_index=None;
        // self.rebuild(0x8001,serial,gen)
    }
    /// 构造通用应答
    pub fn general_response(self,result:u8) -> Self {
        let mut jt808=self;
        let gen= GeneralResponse {
            serial: jt808.serial,
            cmd: jt808.cmd,
            result,
        };
        let body= gen.serialize();
        jt808.cmd=0x8001;
        jt808.body_length = 5;
        jt808.body=body;
        jt808.keep2bit=0;
        jt808.package_flag_1bit=0;
        jt808.encrypt3bit=0;
        jt808.package_total=None;
        jt808.package_index=None;
        jt808
    }

    pub fn set_serial(&mut self,serial: u16){
        self.serial =serial;
    }
    pub fn trans<T: Jt808Trans>(&mut self,vin: &str) -> T {
        T::trans(self,vin)
    }
}
