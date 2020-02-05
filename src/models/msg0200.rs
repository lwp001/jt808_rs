use bytes::{Buf};//Bytes
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use lwp_util::byte_help::{bcd_to_datetime};
use super::{Jt808,Jt808Trans};
use super::schema::gps;

/// 0200 0201 0704 定位数据 Insertable
#[derive(Insertable,Serialize, Deserialize, Debug)]
#[table_name="gps"]
pub struct Position {
    /// 车辆唯一标识
    pub vin: String,
    /// utc beijing - 8h
    pub utc: DateTime<Utc>,
    /// alarm flag
    pub alarm: i32,
    /// status flag
    pub status: i32,
    /// longitude 1.0e-6
    pub lng: f32,
    /// latitude 1.0e-6
    pub lat: f32,
    /// offset_lng
    pub offset_lng: f32,
    /// offset_lat 
    pub offset_lat: f32,
    /// altitude
    pub altitude: i16,
    /// speed 1/10
    pub speed: f32,
    /// direction
    pub direction: i16,
    /// ext_id:0x01 mileage 1/10KM
    pub mileage: Option<f32>,
    /// ext_id:0x02 oil_mass 1/10L
    pub oil_mass: Option<f32>,
    /// ext_id:0x03 r_speed 1/10KM/H
    pub r_speed: Option<f32>,
    /// ext_id:0x04 firm_alarm_id
    pub firm_alarm_id: Option<i16>,
    /// ext_id:0x05 ver_2019 tire pressure
    pub tire_pressure: Option<String>,
    /// ext_id:0x06 ver_2019 carriage temperature
    pub carriage_temperature: Option<i16>,
    /// ext_id:0x25 ext_signal
    pub ext_signal: Option<i32>,
    /// ext_id:0x2A io_status
    pub io_status: Option<i16>,
    /// ext_id:0x2B ad0
    pub ad0: Option<i16>,
    /// ext_id:0x2B ad1
    pub ad1: Option<i16>,
    /// ext_id:0x30 rssi
    pub rssi: Option<i16>,
    /// ext_id:0x31 gnss
    pub gnss: Option<i16>,
    /// extend id:hexstr
    pub ext_data: Option<String>,
    /// version
    pub ver: i16,
}

impl Position {
    fn default() -> Self {
        Position {
            vin: String::new(),
            utc: Utc::now(),
            alarm: 0_i32,
            status: 0_i32,
            lng: 0.0_f32,
            lat: 0.0_f32,
            offset_lng: 0.0_f32,
            offset_lat: 0.0_f32,
            altitude: 0_i16,
            speed: 0.0_f32,
            direction: 0_i16,
            mileage: None,
            oil_mass: None,
            r_speed: None,
            firm_alarm_id: None,
            tire_pressure: None,
            carriage_temperature: None,
            ext_signal: None,
            io_status: None,
            ad0: None,
            ad1: None,
            rssi: None,
            gnss: None,
            ext_data: None,
            ver: 2013,
        }
    }
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

}

impl Jt808Trans for Position  {
    fn trans(jt808:&mut Jt808,vin:&str)->Self {
        let mut position=Position::default();
        position.vin = vin.to_owned();
        let buf= &mut jt808.body;
        position.alarm = buf.get_i32();
        position.status = buf.get_i32();
        position.lng = buf.get_i32() as f32 / 10.0e5;
        position.lat = buf.get_i32() as f32 / 10.0e5;
        // position.offset_lng = ,
        // position.offset_lat: 0.0_f32,
        position.altitude = buf.get_i16();
        position.speed = buf.get_i16() as f32 / 10.0;
        position.direction = buf.get_i16();
        position.utc=bcd_to_datetime(&buf.split_to(6));
        while buf.len() > 0 {
            let cmd = buf.get_u8();
            let len = buf.get_u8() as usize;
            // 需要判断剩余的数据是否小于len
            match cmd {
                0x01 => position.mileage = Some(buf.get_i32() as f32 / 10.0),
                0x02 => position.oil_mass = Some(buf.get_u16() as f32 / 10.0),
                0x03 => position.r_speed = Some(buf.get_u16() as f32 / 10.0),
                0x04=>  position.firm_alarm_id=Some(buf.get_i16()),
                //超速报警 buf.split_to(len);
                0x11 => {buf.advance(len);}
                //进出区域报警
                0x12 => {buf.advance(len);}
                //路段行驶报警
                0x13 => {buf.advance(len);}
                0x25 => position.ext_signal =Some( buf.get_i32()),
                0x2A => position.io_status =Some( buf.get_i16()),
                0x2B => {
                    position.ad1 = Some(buf.get_i16());
                    position.ad0 = Some(buf.get_i16());
                }
                0x30 => position.rssi = Some(buf.get_u8() as i16),
                0x31 => position.gnss = Some(buf.get_u8() as i16),
                _ =>  {buf.advance(len);}
            }
        }
        if jt808.ver.is_some() {
            position.ver = 2019;
        }
        position
    }
}