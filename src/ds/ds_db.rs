use std::{
    sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::Instant,
};
use chrono::Utc;
use concurrent_queue::ConcurrentQueue;
use indexmap::IndexMap;
use sal_sync::services::entity::point::point::Point;
use crate::{
    ds::ds_config::{DsDbConf, PointConf},
    profinet::{
        parse_point::ParsePoint,
        s7::{s7_client::S7Client, s7_parse_bool::S7ParseBool,s7_parse_int::S7ParseInt, s7_parse_real::S7ParseReal},
    },
};
///
/// 
pub(crate) const MAX_QUEUE_SIZE: usize = 1024 * 16;
///
/// 
// #[derive(Debug)]
pub struct DsDb {
    pub name: String,
    pub description: Option<String>,
    pub number: u32,
    pub offset: u32,
    pub size: u32,
    pub delay: u32,
    // pub points: Option<HashMap<String, PointConf>>,
    parse_points: IndexMap<String, Box<dyn ParsePoint>>,
    handle: Option<JoinHandle<()>>,
    cancel: bool,
    sender: Arc<ConcurrentQueue<Point>>,
    pub receiver: Arc<ConcurrentQueue<Point>>,
}
//
//
impl DsDb {
    ///
    /// 
    pub fn new(conf: DsDbConf) -> DsDb {
        let dbg: &str = "DsDb.new | ";
        let _path = conf.name.clone();
        // let mut db_points: IndexMap<String, Box<dyn ParsePoint>> = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        // match config.points.clone() {
        //     None => (),
        //     Some(conf_points) => {
        //         for (point_key, point) in conf_points {
        //             // debug!("\t\t\tdb {:?}: {:?}", pointKey, &point);
        //             let data_type = &point.dataType.clone().unwrap();
        //             if *data_type == "Bool".to_string() {
        //                 db_points.insert(
        //                     point_key.clone(),
        //                     ParsePointType::Bool(
        //                         S7ParsePointBool::new(
        //                             point_key.clone(),
        //                             point_key.clone(),
        //                             point,
        //                         ),
        //                     ),
        //                 );
        //             } else if *data_type == "Int".to_string() {
        //                 db_points.insert(
        //                     point_key.clone(),
        //                     ParsePointType::Int(
        //                         S7ParsePointInt::new(
        //                             point_key.clone(), 
        //                             point_key.clone(), 
        //                             point,
        //                         ),
        //                     ),
        //                 );
        //             } else if *data_type == "Real".to_string() {
        //                 db_points.insert(
        //                     point_key.clone(),
        //                     ParsePointType::Real(
        //                         S7ParsePointReal::new(
        //                             point_key.clone(), 
        //                             point_key.clone(), 
        //                             point,
        //                         ),
        //                     ),
        //                 );
        //             } else {
        //                 log::error!("{} point {:?}: uncnoun data type {:?}", logPref, pointKey, dataType);
        //             }
        //         }
        //     }
        // }
        let sender = Arc::new(ConcurrentQueue::bounded(MAX_QUEUE_SIZE)); 
        DsDb {
            name: conf.name.clone(),
            description: conf.description.clone(),
            number: match conf.number { None => 0, Some(v) => v },
            offset: match conf.offset { None => 0, Some(v) => v },
            size: match conf.size { None => 0, Some(v) => v },
            delay: match conf.delay { None => 0, Some(v) => v },
            // points: conf.points,
            parse_points: Self::configure_parse_points(dbg, 0, &conf),
            handle: None,
            cancel: false,
            sender: sender.clone(),
            receiver: sender,
        }

    }
    ///
    /// Configuring ParsePoint objects depending on point configurations coming from [conf]
    fn configure_parse_points(self_id: &str, tx_id: usize, conf: &DsDbConf) -> IndexMap<String, Box<dyn ParsePoint>> {
        conf.points.iter().map(|(name, point_conf)| {
            match point_conf.data_type.as_str() {
                "Bool" => {
                    (name.to_owned(), Self::box_bool(tx_id, name, point_conf))
                }
                "Int" => {
                    (name.to_owned(), Self::box_int(tx_id, name, point_conf))
                }
                "Real" => {
                    (name.to_owned(), Self::box_real(tx_id, name, point_conf))
                }
                "Double" => {
                    (name.to_owned(), Self::box_real(tx_id, name, point_conf))
                }
                _ => panic!("{}.configureParsePoints | Unknown type '{:?}' for S7 Device", self_id, point_conf.data_type)
            }
        }).collect()
    }
    ///
    ///
    fn box_bool(tx_id: usize, name: &str, conf: &PointConf) -> Box<dyn ParsePoint> {
        Box::new(S7ParseBool::new(tx_id, name.to_owned(), conf))
    }
    ///
    ///
    fn box_int(tx_id: usize, name: &str, conf: &PointConf) -> Box<dyn ParsePoint> {
        Box::new(S7ParseInt::new(
            tx_id,
            name.to_owned(),
            conf,
            Self::int_filter(conf.filters.clone()),
        ))
    }
    ///
    ///
    fn box_real(tx_id: usize, name: &str, conf: &PointConf) -> Box<dyn ParsePoint> {
        Box::new(S7ParseReal::new(
            tx_id,
            name.to_owned(),
            conf,
            Self::real_filter(conf.filters.clone()),
        ))
    }
    ///
    ///
    pub fn run(this: Arc<Mutex<Self>>, client: S7Client) {
        const dbg: &str = "DsDb.run | ";
        log::info!("{} starting in thread: {:?}...", dbg, thread::current().name().unwrap());
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
                if let Ok(is_connected) = client.is_connected() {
                    if is_connected {
                        log::trace!("{} reading DB: {:?}, offset: {:?}, size: {:?}", dbg, me.number, me.offset, me.size);
                        match client.read(me.number, me.offset, me.size) {
                            Ok(bytes) => {
                                log::trace!("{}.read | bytes: {:?}", dbg, bytes);
                                let mut message = String::new();
                                for (_, parse_point) in &mut me.parse_points {
                                    if let Some(point) = parse_point.next(&bytes, Utc::now()) {
                                        // log::debug!("{}.read | point: {:?}", self.id, point);
                                        match sender.push(point) {
                                            Ok(_) => {}
                                            Err(err) => {
                                                message = format!("{}.read | send error: {}", dbg, err);
                                                log::warn!("{}", message);
                                            }
                                        }
                                    }
                                }
                                match message.is_empty() {
                                    true => Ok(()),
                                    false => Err(message),
                                };
                            }
                            Err(err) => {
                                log::error!("{} client.read error: {}", dbg, err);
                                std::thread::sleep(std::time::Duration::from_millis((delay * 100) as u64));
                            },
                        }
                    }
                } else {
                    log::error!("{} wait for connection...", dbg);
                    std::thread::sleep(std::time::Duration::from_millis((delay * 100) as u64));
                }
                let dt = Instant::now() - t;
                // debug!("{} {:?} elapsed: {:?} ({:?})", logPref, me.name , dt, dt.as_millis());
                let wait: i128 = (delay as i128) - (dt.as_millis() as i128);
                if wait > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(wait as u64));
                }
                let dt = Instant::now() - t;
                log::trace!("{} {:?} elapsed: {:?} ({:?})", dbg, me.name , dt, dt.as_millis());
            }
            log::info!("{} exit", dbg);
        }).unwrap();
        me1.lock().unwrap().handle = Some(handle);
        log::info!("{} started", dbg);
    }
}
