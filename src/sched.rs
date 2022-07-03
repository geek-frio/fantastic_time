use crate::img::*;
use crate::store::{ImgMetaDO, ImgMetaDao};
use chrono::NaiveDateTime;
use crossbeam::channel::Receiver as BroadReceiver;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;
use tracing::info;

#[derive(Clone)]
pub struct ImgHandle {
    sender: Sender<HandleEvent>,
}

impl ImgHandle {
    fn new(sender: Sender<HandleEvent>, shutdown: BroadReceiver<()>) -> ImgHandle {
        let handle = ImgHandle {
            sender: sender.clone(),
        };

        std::thread::spawn(move || loop {
            if let Ok(_) = shutdown.try_recv() {
                info!("Has found shutdown signal, exit!");
            }

            sender.send(HandleEvent::Tick).expect("Sent tick failed!");
            std::thread::sleep(Duration::from_secs(1));
        });

        handle
    }
}

impl ImgHandle {
    pub fn sched(&self, event: HandleEvent) {
        self.sender.send(event).expect("Sched handle event failed!");
    }
}

pub enum HandleEvent {
    Tick,
    NewPathEvent(PathBuf),
}

pub struct ImgActor {
    receiver: Receiver<HandleEvent>,
    dao: Arc<ImgMetaDao>,
}

impl From<ImgMeta> for ImgMetaDO {
    fn from(img: ImgMeta) -> ImgMetaDO {
        let timestamp = chrono::offset::Local::now().timestamp();

        let time = img
            .time
            .unwrap_or(NaiveDateTime::from_timestamp(timestamp, 0));
        let sign = img.sig.unwrap();

        ImgMetaDO { id: 0, time, sign }
    }
}

impl ImgActor {
    pub fn run(self) {
        let receiver = self.receiver;
        let dao = self.dao;
        std::thread::spawn(move || {
            let mut batch = Vec::new();
            loop {
                let event = receiver.recv().expect("接收图片处理任务失败");

                match event {
                    HandleEvent::NewPathEvent(path) => {
                        batch.push(path);

                        if batch.len() > 500 {
                            Self::batch_write(batch, &dao);
                            batch = Vec::new();
                        }
                    }

                    HandleEvent::Tick => {
                        if batch.len() > 0 {
                            Self::batch_write(batch, &dao);
                            batch = Vec::new();
                        }
                    }
                }
            }
        });
    }

    fn batch_write(batch: Vec<PathBuf>, dao: &Arc<ImgMetaDao>) {
        let dos = batch
            .into_iter()
            .map(|p| {
                let res = retrive_img_datetime(p.as_path());

                match res {
                    Ok(img_data) => {
                        let img = <ImgMetaDO as From<ImgMeta>>::from(img_data);
                        Some(img)
                    }

                    Err(_) => None,
                }
            })
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect::<Vec<ImgMetaDO>>();

        let r = dao.batch_write(dos);

        if let Err(dos) = r {
            error!("记录写入失败:{:?}", dos);
        }
    }
}
