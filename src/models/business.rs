
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisVehicle<'a> {
    pub vin: &'a str,
    pub lpn: &'a str,
    /// 行政区划
    pub division_code:i32,
    pub org_id: i64,
    pub auth_code: i64,
}

impl<'a> RedisVehicle<'a> {
    pub fn get_channel(&self,sim: &str) -> String {
        format!("{}:{}:{}",self.division_code / 100,self.org_id,sim)
    }
}