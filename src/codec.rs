use bytes::{BufMut,BytesMut,Bytes};//Buf
use tokio_util::codec::{Decoder,Encoder};
use std::{ fmt, io, usize}; //cmp,str
const JT808_FLAG: u8 = 0x7E;

/// jt808 特性
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Jt808Codec {
    /// 从哪里开始查找结束字符
    next_index: usize,
    /// 最长长度 超过后判断不合法
    max_length: usize,
    /// 最短长度 小于后判断不合法
    min_length: usize,
}

impl Jt808Codec {
    ///  创建 jt808 
    pub fn build(max_length: usize,min_length: usize)-> Jt808Codec {
        Jt808Codec {
            next_index: 1,
            max_length,
            min_length,
        }
    }
    /// 默认jt808 
    pub fn new() -> Jt808Codec {
        Jt808Codec::build(2048,14)
    }
}

/// 反转义
fn unescape(buf: &[u8]) -> Bytes {
    let mut data = BytesMut::with_capacity(buf.len());
    let mut priv_key:u8=0;
    for b in buf.iter() {
        if priv_key==0x7D_u8
        {
            data.put_u8(0x7C+b);
            priv_key=0;
            continue;
        }
        if *b == 0x7Du8 {
            priv_key=0x7D;
            continue;
        }
        data.put_u8(*b);
    }
    data.freeze()
}

impl Decoder for Jt808Codec {
    type Item = Bytes;
    type Error = Jt808CodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>, Jt808CodecError> {
        // loop {   }
        if buf.len() < 1 {
            //无数据唤醒 以后记录分析
            return Ok(None);
        }
        //无效数据
        if buf[0] != JT808_FLAG {
            return Err(Jt808CodecError::NoProtocol(buf.split_to(buf.len())));
        }
        let new_offset = buf[self.next_index..buf.len()]
            .iter()
            .position(|b| *b == JT808_FLAG);
        match new_offset {
            Some(offset) =>{
                let new_index = offset + self.next_index;
                //重置查找位置
                self.next_index = 1;
                if  new_index >= self.min_length {
                    let jt808_frame = buf.split_to(new_index + 1);
                    let frame=unescape(&jt808_frame[..]);
                    return Ok(Some(frame));
                } else {
                    // 有帧头帧尾但数据长度不对 所以是非协议数据
                    return Err(Jt808CodecError::NoProtocol(buf.split_to(buf.len())));
                }
            }
            None if buf.len() > self.max_length  =>
                {
                    //不合法数据 数据长度已超出
                    self.next_index = 1;
                    return Err(Jt808CodecError::MaxLengthExceeded(buf.split_to(buf.len())));
                }
            None =>{
                //数据帧不完整等待下一帧
                self.next_index=buf.len();
                return Ok(None);
            }
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>, Jt808CodecError> {
        Ok(match self.decode(buf)? {
            Some(frame) => Some(frame),
            None => None,
        })
    }
}

fn escape(b:u8,buf: &mut BytesMut){
    if b == 0x7Du8 || b==0x7E_u8 {
        buf.put_u8(0x7D_u8);
        buf.put_u8(b - 0x7C_u8);
    }else {
        buf.put_u8(b);
    }
}

impl Encoder for Jt808Codec {
    type Item = Bytes;
    type Error = io::Error;

    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), io::Error> {     
        buf.reserve(data.len()+10);
        buf.put_u8(0x7E);
        // 转义
        for b in data.iter() {
            escape(*b,buf);
        }
        buf.put_u8(0x7E);
        Ok(())
    }
}


impl Default for Jt808Codec {
    fn default() -> Self {
        Self::new()
    }
}

/// An error occured while encoding or decoding a line.
#[derive(Debug)]
pub enum Jt808CodecError {
    /// The maximum line length was exceeded.
    MaxLengthExceeded(BytesMut),
    ///非协议数据
    NoProtocol(BytesMut),
    /// An IO error occured.
    Io(io::Error),
}

impl fmt::Display for Jt808CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Jt808CodecError::MaxLengthExceeded(ref err_data) => write!(f, "data exceeded max length jt808: {:X}",err_data),
            Jt808CodecError::NoProtocol(ref err_data)=> write!(f,"not jt808 protocol data {:X}",err_data),
            Jt808CodecError::Io(e) => write!(f, "{:?}", e),
        }
    }
}

impl From<io::Error> for Jt808CodecError {
    fn from(e: io::Error) -> Jt808CodecError {
        Jt808CodecError::Io(e)
    }
}

impl std::error::Error for Jt808CodecError {}


