use std::{
    sync::{Arc, Mutex},
    collections::HashMap,
};
use crate::{
    profinet::s7::s7_client::S7Client,
    ds::{
        ds_db::DsDb,
        ds_config::DsIedConf, 
    },
};

// #[derive(Debug)]
pub struct DsIed {
    pub name: String,
    pub description: Option<String>,
    pub ip: String,
    pub rack: u32,
    pub slot: u32,
    pub dbs: HashMap<String, Arc<Mutex<DsDb>>>,
}
impl DsIed {
    ///
    pub fn new(
        config: DsIedConf,
    ) -> DsIed {
        let _path = config.name.clone();
        let mut dbs: HashMap<String, Arc<Mutex<DsDb>>> = HashMap::new();
        match config.dbs.clone() {
            None => (),
            Some(conf_dbs) => {
                for (db_key, db_conf) in conf_dbs {
                    let db = Arc::new(Mutex::new(DsDb::new(db_conf)));
                    // debug!("\t\tdb {:?}: {:#?}", dbKey, db);
                    dbs.insert(
                        db_key, 
                        db,
                    );
                }
            }
        }
        DsIed {
            name: config.name,
            description: config.description,
            ip: match config.ip { None => String::new(), Some(v) => v },
            rack: match config.rack { None => 0, Some(v) => v },
            slot: match config.slot { None => 0, Some(v) => v },
            dbs: dbs,
        }

    }
    ///
    pub fn run(&mut self) {
        let dbg = "[DsIed.run]";
        for (_key, db) in &self.dbs {
            let mut client = S7Client::new(
                &self.name,
                self.ip.clone(),
            );
            log::debug!("{} client: {:#?}", dbg, client);
            client.connect();
            // debug!("{} client: {:#?}", logPref, client);
            DsDb::run(db.clone(), client);
        }
    }
}

