
use std::cell::RefCell;

use crate::domain::backend::ports::Loader;
use crate::domain::backend::models::*;
use anyhow::*;
use tokio::runtime::Runtime;
use tokio::sync::watch::{Sender,Receiver};
use url::Url;

struct Inner {
    url: Receiver<Option<Url>>,
    response: Option<reqwest::Response>,
}
pub struct TokioReqwestLoader{
    rt: Runtime,
    next_url: Sender<Option<Url>>,
    inner: RefCell<Inner>,
}

impl Loader for TokioReqwestLoader {
    fn new() -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build().unwrap();
        let (tx,rx) = tokio::sync::watch::channel(None);
        Self {
            rt,
            next_url: tx,
            inner: RefCell::new(Inner { url: rx, response: None }),
        }
    }

    fn read_chunk(&self) -> Result<Option<Chunk>> {
        log::trace!("read_chunk()");
        self.rt.block_on(async {
            let mut inner = self.inner.borrow_mut();

            // if there is no current response to read data from, 
            // wait for a new Url to be set and receive a response
            // from which we can read chunks
            let new_url_set = inner.url.has_changed()?;
            if new_url_set {
                let url = inner.url.borrow_and_update().clone();
                if let Some(url) = url {
                    inner.response = Some(reqwest::get(url).await?);
                } else {
                    inner.response = None;
                }
            }

            if inner.response.is_none() {
                log::trace!("no stream connection, no data to load");
                return Ok(None);
            }

            // read next chunk of data
            log::trace!("read chunk from stream");
            let response = inner.response.as_mut().unwrap();
            match response.chunk().await? {
                Some(chunk) => Ok(Some(chunk.into())),
                None => {
                    inner.response = None;
                    Ok(None)
                },
            } 
        })

    }

    fn set_url(&mut self, url: Option<url::Url>) {
        log::trace!("new url set: {url:?}");
        self.next_url.send(url).unwrap();
    }
}