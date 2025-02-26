use super::KeyValueBackend;
use crate::types::HeaderIndexView;
use ckb_types::{packed::Byte32, prelude::*};
use sled::{Config, Db, Mode};
use std::path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tempfile::TempDir;

pub(crate) struct SledBackend {
    count: AtomicUsize,
    db: Db,
    _tmpdir: TempDir,
}

impl KeyValueBackend for SledBackend {
    fn new<P>(tmp_path: Option<P>) -> Self
    where
        P: AsRef<path::Path>,
    {
        let mut builder = tempfile::Builder::new();
        builder.prefix("ckb-tmp-");
        let tmpdir = if let Some(ref path) = tmp_path {
            builder.tempdir_in(path)
        } else {
            builder.tempdir()
        }
        .expect("failed to create a tempdir to save header map into disk");

        // use a smaller system page cache here since we are using sled as a temporary storage,
        // most of the time we will only read header from memory.
        let db: Db = Config::new()
            .mode(Mode::HighThroughput)
            .cache_capacity(64 * 1024 * 1024)
            .path(tmpdir.path())
            .open()
            .expect("failed to open a key-value database to save header map into disk");

        Self {
            db,
            _tmpdir: tmpdir,
            count: AtomicUsize::new(0),
        }
    }

    fn len(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    fn contains_key(&self, key: &Byte32) -> bool {
        self.db
            .contains_key(key.as_slice())
            .expect("sled contains_key")
    }

    fn get(&self, key: &Byte32) -> Option<HeaderIndexView> {
        self.db
            .get(key.as_slice())
            .unwrap_or_else(|err| panic!("read header map from disk should be ok, but {err}"))
            .map(|slice| HeaderIndexView::from_slice_should_be_ok(key.as_slice(), slice.as_ref()))
    }

    fn insert(&self, value: &HeaderIndexView) -> Option<()> {
        let key = value.hash();
        let last_value = self
            .db
            .insert(key.as_slice(), value.to_vec())
            .expect("failed to insert item to sled");
        if last_value.is_none() {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        last_value.map(|_| ())
    }

    fn insert_batch(&self, values: &[HeaderIndexView]) {
        let mut count = 0;
        for value in values {
            let key = value.hash();
            let last_value = self
                .db
                .insert(key.as_slice(), value.to_vec())
                .expect("failed to insert item to sled");
            if last_value.is_none() {
                count += 1;
            }
        }
        self.count.fetch_add(count, Ordering::SeqCst);
    }

    fn remove(&self, key: &Byte32) -> Option<HeaderIndexView> {
        let old_value = self
            .db
            .remove(key.as_slice())
            .expect("failed to remove item from sled");

        old_value.map(|slice| {
            self.count.fetch_sub(1, Ordering::SeqCst);
            HeaderIndexView::from_slice_should_be_ok(key.as_slice(), &slice)
        })
    }

    fn remove_no_return(&self, key: &Byte32) {
        let old_value = self
            .db
            .remove(key.as_slice())
            .expect("failed to remove item from sled");
        if old_value.is_some() {
            self.count.fetch_sub(1, Ordering::SeqCst);
        }
    }
}
