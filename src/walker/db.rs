use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier, Mutex,
    },
};

use anyhow::Error as AnyError;
use rusqlite::{Connection, Result};

#[derive(Clone)]
struct Db {
    conns: Arc<Mutex<Vec<WrapConnection>>>,
    barrier: Arc<Barrier>,
    is_init: Arc<AtomicBool>,
    cap: usize,
}

enum WrapConnection {
    Conn(Option<Connection>, Db),
    Empty,
}

impl Drop for WrapConnection {
    fn drop(&mut self) {
        }

        // match old {
        //     WrapConnection::Conn(conn, db) => {
        //         db.return_conn(conn);
        //     }
        //     _ => {}
        // }
    }
}

impl Db {
    fn new(cap: usize) -> Db {
        let mut vec = vec![];
        Db {
            conns: Arc::new(Mutex::new(vec)),
            barrier: Arc::new(Barrier::new(cap)),
            is_init: Arc::new(AtomicBool::new(false)),
            cap,
        }
    }

    fn init<P: AsRef<Path>>(&self, path: &P) {
        let mut guard = self.conns.lock().expect("Lock self conns failed!");
        for _ in 0..self.cap {
            let conn = Connection::open(path).expect("Open connection of sqlite failed!");
            guard.push(WrapConnection::Conn(Some(conn), self.clone()));
        }
        self.is_init.store(true, Ordering::Relaxed);
    }

    fn take_conn(&self) -> Result<Option<WrapConnection>, AnyError> {
        self.barrier.wait();
        let mut guard = self.conns.lock().unwrap();
        let conn = guard.pop();
        return Ok(conn);
    }

    fn return_conn(&self, conn: Connection) {
        let mut guard = self.conns.lock().expect("Lock self conns failed!");
        guard.push(WrapConnection::Conn(Some(conn), self.clone()));
    }
}
