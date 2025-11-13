use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{Service, entity::{Name, Object, Point}}, sync::{Handles, Owner, RwLock, channel::{self, RecvTimeoutError, Sender}}};
use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self}, time::Duration};
///
/// 
pub struct Receiver {
    dbg: Dbg,
    name: Name,
    rx_send: Sender<Point>,
    rx_recv: Owner<channel::Receiver<Point>>,
    tx_send: Owner<channel::Sender<u16>>,
    buf: Arc<RwLock<Vec<u16>>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl Receiver {
    ///
    /// Creates new instance [Receiver]
    /// - `index` - Index of instance (TaskTestReceiver1, TaskTestReceiver2,...etc)
    /// - `recv_queue` - name of the link used for receiving Point's
    /// - `iterations` - count down with each received Point, when zero Receiver exits
    pub fn new(parent: &str) -> Self {
        let (rx_send, recv): (Sender<Point>, channel::Receiver<Point>) = channel::unbounded();
        let name = Name::new(parent, format!("Receiver"));
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            rx_send,
            rx_recv: Owner::new(recv),
            tx_send: Owner::empty(),
            buf: Arc::new(RwLock::new(vec![])),
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Returns stream of collected values in vector
    pub fn values(&self) -> channel::Receiver<u16> {
        let (send, recv) = channel::unbounded();
        self.tx_send.replace(send);
        recv
    }
}
//
// 
impl Object for Receiver {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for Receiver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Receiver")
            .field("id", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for Receiver {
    //
    //
    fn get_link(&self, _: &str) -> Sender<Point> {
        self.rx_send.clone()
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        log::info!("{}.run | Starting...", dbg);
        let exit = self.exit.clone();
        let rx_recv = self.rx_recv.take().unwrap();
        let handle = thread::Builder::new().name(dbg.to_string()).spawn(move || {
            // log::info!("Task({}).run | prepared", name);
            'main: loop {
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
                match rx_recv.recv_timeout(Duration::from_millis(100)) {
                    Ok(point) => {
                        log::trace!("{}.run | received Point: {:#?}", dbg, point);
                        // debug!("{}.run | value: {}\treceived SQL: {:?}", value, sql);
                        match point {
                            Point::Bool(_) => {},
                            Point::Int(_) => {},
                            Point::Real(_) => {},
                            Point::Double(_) => {},
                            Point::String(p) => {
                                if p.name.to_lowercase().ends_with("exit") || p.value == "exit" {
                                    break 'main;
                                }
                            },
                            Point::Bytes(_) => {},
                        }
                    }
                    Err(err) => {
                        match err {
                            RecvTimeoutError::Timeout => {},
                            _ => log::error!("{}.run | Error receiving from queue: {:?}", dbg, err),
                        }
                        // error_count += 1;
                        // if errorCount > 10 {
                        //     log::warn!("{}.run | Error receiving count > 10, exit...", self_id);
                        //     break 'inner;
                        // }        
                    }
                };
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
            };
            log::info!("{}.run | exit", dbg);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                Ok(())
                        }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Relaxed);
    }
    // pub fn getInputValues(&mut self) -> Receiver<PointType> {
    //     self.recv.pop().unwrap()
    // }
}
