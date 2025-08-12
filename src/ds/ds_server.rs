use std::{
    collections::HashMap, 
    thread::{self}, sync::Arc,
};

use concurrent_queue::ConcurrentQueue;
use log::{
    info,
    debug,
    // error,
};

use crate::ds::{
    ds_config::DsConfig, 
    ds_line::DsLine, ds_point::DsPoint,
};

///
/// 
#[derive(Debug)]
pub struct DsServer {
    pub name: String,
    pub description: Option<String>,
    pub config: DsConfig,
    lines: HashMap<String, DsLine>,
    pub queues: Vec<Arc<ConcurrentQueue<DsPoint>>>,
    // handle: Option<JoinHandle<()>>,
    // cancel: bool,
    // sender: Arc<ConcurrentQueue<DsPoint>>,
    // pub receiver: Arc<ConcurrentQueue<DsPoint>>,        
}
impl DsServer {
    ///
    pub fn new(
    ) -> DsServer {
        let dir = std::env::current_dir().unwrap();
        let path: &str = &format!("{}/conf.json", dir.to_str().unwrap());
        let config = DsConfig::new(path.to_string());
        DsServer {
            name: "DsServer".to_string(),   // config.name
            description: Some("DsServer".to_string()), // config.description,
            config: config,
            lines: HashMap::new(),
            queues: vec![],
            // handle: None,
            // cancel: false,
            // sender: sender.clone(),
            // receiver: sender,                
        }

    }
    ///
    // fn read() {

    // }
    ///
    pub fn run(&mut self) {
        let dbg = "DsServer.run";
        info!("{} starting in thread: {:?}...", dbg, thread::current().name().unwrap());
        // let mut receivers: Vec<Arc<ConcurrentQueue<DsPoint>>>  = vec![];
        for (line_key, line_conf) in &(self.config.lines) {
            debug!("{} line {:?}: ", dbg, line_key);
            let mut line = DsLine::new((*line_conf).clone());
            for (_ied_key, ied) in &line.ieds {
                for (_db_key, db) in &ied.dbs {
                    let rcv = &db.lock().receiver;
                    self.queues.push(rcv.clone());
                }
            }
            line.run();
            self.lines.insert(
                line_key.clone(), 
                line,
            );
        }
        info!("{} all lines started", dbg);
    }
}
