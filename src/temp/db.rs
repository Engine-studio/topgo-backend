use std::collections::HashMap;

use r2d2_redis::{RedisConnectionManager, r2d2, redis::{self, Commands}};
use actix_web_dev::error::{
    Result,
    ApiError,
    ErrorType,
};
use serde::{Serialize,Deserialize};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Coords {
    lat: f64,
    lng: f64,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct CourierLocation {
    pub courier_id: i64,
    location: Coords,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct CoordsWithStamp {
    courier_id: i64,
    lat: f64,
    lng: f64,
    timestamp: i64,
}

pub async fn set_coords(loc: CourierLocation, conn: &mut redis::Connection) -> Result<()> {
    conn.hset("courier", loc.courier_id, serde_json::to_string(&CoordsWithStamp {
            courier_id: loc.courier_id,
            lat: loc.location.lat,
            lng: loc.location.lng,
            timestamp: chrono::Utc::now().timestamp(),
    }).expect("err rerealize"))?;
    Ok(())
}

pub async fn get_coords(
    conn: &mut redis::Connection
) -> Result<Vec<CoordsWithStamp>> {
    let v: Vec<(i64,String)> = conn.hgetall("courier")?;
    println!("v: {:?}",v);
    let r: Vec<CoordsWithStamp> = v
        .into_iter()
        .map(|(_,f)|{
            serde_json::from_str(&f).unwrap()
        })
        .collect();
    println!("r: {:?}",r);
    Ok(r)
}

pub async fn rm_coords(
    courier_id: i64,
    conn: &mut redis::Connection
) -> Result<()> {
    conn.hdel("courier", courier_id)?;
    Ok(())
}

