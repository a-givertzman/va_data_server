use std::{
    collections::{HashMap, BTreeMap}, 
    thread::{self, JoinHandle}, sync::{Arc, Mutex}, 
    time::Instant
};
use concurrent_queue::ConcurrentQueue;
use indexmap::IndexMap;
use crate::{
    ds::{
        ds_config::{DsDbConf, PointConf}, 
        ds_point::DsPoint, ds_status::DsStatus,
    }, profinet::{parse_point::ParsePoint, s7::s7_client::S7Client}
};
pub(crate) const MAX_QUEUE_SIZE: usize = 1024 * 16;

#[derive(Debug)]
pub struct DsDb {
    pub name: String,
    pub description: Option<String>,
    pub number: u32,
    pub offset: u32,
    pub size: u32,
    pub delay: u32,
    pub points: Option<HashMap<String, PointConf>>,
    parse_points: IndexMap<String, Box<dyn ParsePoint>>,
    handle: Option<JoinHandle<()>>,
    cancel: bool,
    sender: Arc<ConcurrentQueue<DsPoint>>,
    pub receiver: Arc<ConcurrentQueue<DsPoint>>,
}
impl DsDb {
    ///
    pub fn new(config: DsDbConf) -> DsDb {
        let dbg: &str = "DsDb.new | ";
        let _path = config.name.clone();
        let mut db_points: BTreeMap<String, ParsePointType> = BTreeMap::new();
        match config.points.clone() {
            None => (),
            Some(conf_points) => {
                for (point_key, point) in conf_points {
                    // debug!("\t\t\tdb {:?}: {:?}", pointKey, &point);
                    let data_type = &point.dataType.clone().unwrap();
                    if *data_type == "Bool".to_string() {
                        db_points.insert(
                            point_key.clone(),
                            ParsePointType::Bool(
                                S7ParsePointBool::new(
                                    point_key.clone(),
                                    point_key.clone(),
                                    point,
                                ),
                            ),
                        );
                    } else if *data_type == "Int".to_string() {
                        db_points.insert(
                            point_key.clone(),
                            ParsePointType::Int(
                                S7ParsePointInt::new(
                                    point_key.clone(), 
                                    point_key.clone(), 
                                    point,
                                ),
                            ),
                        );
                    } else if *data_type == "Real".to_string() {
                        db_points.insert(
                            point_key.clone(),
                            ParsePointType::Real(
                                S7ParsePointReal::new(
                                    point_key.clone(), 
                                    point_key.clone(), 
                                    point,
                                ),
                            ),
                        );
                    } else {
                        log::error!("{} point {:?}: uncnoun data type {:?}", logPref, pointKey, dataType);
                    }
                }
            }
        }
        let sender = Arc::new(ConcurrentQueue::bounded(MAX_QUEUE_SIZE)); 
        DsDb {
            name: config.name,
            description: config.description,
            number: match config.number { None => 0, Some(v) => v },
            offset: match config.offset { None => 0, Some(v) => v },
            size: match config.size { None => 0, Some(v) => v },
            delay: match config.delay { None => 0, Some(v) => v },
            points: config.points,  // Some(localPoints),
            parse_points: db_points,
            handle: None,
            cancel: false,
            sender: sender.clone(),
            receiver: sender,
        }

    }
    ///
    /// Configuring ParsePoint objects depending on point configurations coming from [conf]
    fn configure_parse_points(self_id: &str, tx_id: usize, conf: &ProfinetDbConfig) -> IndexMap<String, Box<dyn ParsePoint>> {
        conf.points.iter().map(|point_conf| {
            match point_conf.type_ {
                PointConfigType::Bool => {
                    (point_conf.name.clone(), Self::box_bool(tx_id, point_conf.name.clone(), point_conf))
                }
                PointConfigType::Int => {
                    (point_conf.name.clone(), Self::box_int(tx_id, point_conf.name.clone(), point_conf))
                }
                PointConfigType::Real => {
                    (point_conf.name.clone(), Self::box_real(tx_id, point_conf.name.clone(), point_conf))
                }
                PointConfigType::Double => {
                    (point_conf.name.clone(), Self::box_real(tx_id, point_conf.name.clone(), point_conf))
                }
                _ => panic!("{}.configureParsePoints | Unknown type '{:?}' for S7 Device", self_id, point_conf.type_)
            }
        }).collect()
    }
    ///
    ///
    pub fn run(this: Arc<Mutex<Self>>, client: S7Client) {
        const logPref: &str = "[DsDb.run]";
        log::info!("{} starting in thread: {:?}...", logPref, thread::current().name().unwrap());
        // let h = &mut self.handle;
        let me = this.clone();
        let me1 = this.clone();
        let delay = this.clone().lock().unwrap().delay as u64;
        let handle = thread::Builder::new().name("DsDb.thread".to_string()).spawn(move || {
            let sender = me.clone().lock().unwrap().sender.clone();
            while !me.clone().lock().unwrap().cancel {
                let me = me.lock().unwrap();
                let t = Instant::now();
                // let t = Utc::now();
                if client.isConnected {
                    log::trace!("{} reading DB: {:?}, offset: {:?}, size: {:?}", logPref, me.number, me.offset, me.size);
                    match client.read(me.number, me.offset, me.size) {
                        Ok(bytes) => {
                            // let bytes = client.read(899, 0, 34).unwrap();
                            // print!("\x1B[2J\x1B[1;1H");
                            // debug!("{:?}", bytes);
                            for (_key, pointType) in &me.parse_points {
                                match pointType.clone() {
                                    ParsePointType::Bool(mut point) => {
                                        point.addRaw(&bytes);
                                        // debug!("{} parsed point Bool: {:?}", logPref, point);
                                        if point.isChanged() {
                                            let dsPoint = DsPoint::newBool(
                                                point.name.as_str(),
                                                false,
                                                DsStatus::Ok,
                                                point.timestamp,
                                                point.h,
                                                point.a,
                                            );
                                            // debug!("{} point (Bool): {:?} {:?}", logPref, dsPoint.name, dsPoint.value);
                                            sender.push(dsPoint).unwrap()
                                        }
                                    },
                                    ParsePointType::Int(mut point) => {
                                        point.addRaw(&bytes);
                                        // debug!("{} parsed point Int: {:?}", logPref, point);
                                        if point.isChanged() {
                                            let dsPoint = DsPoint::newInt(
                                                point.name.as_str(),
                                                0,
                                                DsStatus::Ok,
                                                point.timestamp,
                                                point.h,
                                                point.a,
                                            );
                                            // debug!("{} point (Int): {:?} {:?}", logPref, dsPoint.name, dsPoint.value);
                                            sender.push(dsPoint).unwrap()
                                        }
                                    },
                                    ParsePointType::Real(mut point) => {
                                        point.addRaw(&bytes);
                                        // debug!("{} parsed point Real: {:?}", logPref, point);
                                        if point.isChanged() {
                                            let dsPoint = DsPoint::newReal(
                                                point.name.as_str(),
                                                point.value,
                                                DsStatus::Ok,
                                                point.timestamp,
                                                point.h,
                                                point.a,
                                            );
                                            // debug!("{} point (Real): {:?} {:?}", logPref, dsPoint.name, dsPoint.value);
                                            sender.push(dsPoint).unwrap();
                                        }
                                    },
                                }
                            }
                        }        
                        Err(err) => {
                            log::error!("{} client.read error: {}", logPref, err);
                            std::thread::sleep(std::time::Duration::from_millis((delay * 100) as u64));
                        },
                    }
                } else {
                    log::error!("{} wait for connection...", logPref);
                    std::thread::sleep(std::time::Duration::from_millis((delay * 100) as u64));
                }
                let dt = Instant::now() - t;
                // debug!("{} {:?} elapsed: {:?} ({:?})", logPref, me.name , dt, dt.as_millis());
                let wait: i128 = (delay as i128) - (dt.as_millis() as i128);
                if wait > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(wait as u64));
                }
                let dt = Instant::now() - t;
                log::trace!("{} {:?} elapsed: {:?} ({:?})", logPref, me.name , dt, dt.as_millis());
            }
            log::info!("{} exit", logPref);
        }).unwrap();
        me1.lock().unwrap().handle = Some(handle);
        log::info!("{} started", logPref);
    }
}
