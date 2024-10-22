use crate::{
    errors::{Error, PeerStoreError},
    peer_store::{
        addr_manager::AddrManager,
        ban_list::BanList,
        types::{AddrInfo, BannedAddr},
        PeerStore,
    },
};
use ckb_logger::{debug, error};
use std::path::Path;
use std::{
    fs::{copy, create_dir_all, remove_file, rename, File, OpenOptions},
    io::{Read, Write},
};

const DEFAULT_ADDR_MANAGER_DB: &str = "addr_manager.db";
const DEFAULT_BAN_LIST_DB: &str = "ban_list.db";

impl AddrManager {
    /// Load address list from disk
    pub fn load<R: Read>(r: R) -> Result<Self, Error> {
        let addrs: Vec<AddrInfo> = serde_json::from_reader(r).map_err(PeerStoreError::Serde)?;
        let mut addr_manager = AddrManager::default();
        addrs.into_iter().for_each(|addr| addr_manager.add(addr));
        Ok(addr_manager)
    }

    /// Dump address list to disk
    pub fn dump(&self, mut file: File) -> Result<(), Error> {
        let addrs: Vec<_> = self.addrs_iter().collect();
        debug!("Dump {} addrs", addrs.len());
        // empty file and dump the json string to it
        file.set_len(0)
            .and_then(|_| serde_json::to_string(&addrs).map_err(Into::into))
            .and_then(|json_string| file.write_all(json_string.as_bytes()))
            .and_then(|_| file.sync_all())
            .map_err(Into::into)
    }

    #[cfg(target_family = "wasm")]
    pub fn dump_with_fn<F: Fn(&[u8])>(&self, f: F) {
        let addrs: Vec<_> = self.addrs_iter().collect();
        debug!("Dump {} addrs", addrs.len());
        let bytes = serde_json::to_string(&addrs).unwrap();
        f(bytes.as_bytes())
    }
}

impl BanList {
    /// Load ban list from disk
    pub fn load<R: Read>(r: R) -> Result<Self, Error> {
        let banned_addrs: Vec<BannedAddr> =
            serde_json::from_reader(r).map_err(PeerStoreError::Serde)?;
        let mut ban_list = BanList::default();
        banned_addrs
            .into_iter()
            .for_each(|banned_addr| ban_list.ban(banned_addr));
        Ok(ban_list)
    }

    /// Dump ban list to disk
    pub fn dump(&self, mut file: File) -> Result<(), Error> {
        let banned_addrs = self.get_banned_addrs();
        debug!("Dump {} banned addrs", banned_addrs.len());
        // empty file and dump the json string to it
        file.set_len(0)
            .and_then(|_| serde_json::to_string(&banned_addrs).map_err(Into::into))
            .and_then(|json_string| file.write_all(json_string.as_bytes()))
            .and_then(|_| file.sync_all())
            .map_err(Into::into)
    }

    #[cfg(target_family = "wasm")]
    pub fn dump_with_fn<F: Fn(&[u8])>(&self, f: F) {
        let addrs: Vec<_> = self.get_banned_addrs();
        debug!("Dump {} banned addrs", addrs.len());
        let bytes = serde_json::to_string(&addrs).unwrap();
        f(bytes.as_bytes())
    }
}

impl PeerStore {
    /// Init peer store from disk
    pub fn load_from_dir_or_default<P: AsRef<Path>>(path: P) -> Self {
        let addr_manager_path = path.as_ref().join(DEFAULT_ADDR_MANAGER_DB);
        let ban_list_path = path.as_ref().join(DEFAULT_BAN_LIST_DB);

        let addr_manager = File::open(&addr_manager_path)
            .map_err(|err| {
                debug!(
                    "Failed to open AddrManager db, file: {:?}, error: {:?}",
                    addr_manager_path, err
                )
            })
            .and_then(|file| {
                AddrManager::load(std::io::BufReader::new(file)).map_err(|err| {
                    error!(
                        "Failed to load AddrManager db, file: {:?}, error: {:?}",
                        addr_manager_path, err
                    )
                })
            })
            .unwrap_or_default();

        let ban_list = File::open(&ban_list_path)
            .map_err(|err| {
                debug!(
                    "Failed to open BanList db, file: {:?}, error: {:?}",
                    ban_list_path, err
                )
            })
            .and_then(|file| {
                BanList::load(std::io::BufReader::new(file)).map_err(|err| {
                    error!(
                        "Failed to load BanList db, file: {:?}, error: {:?}",
                        ban_list_path, err
                    )
                })
            })
            .unwrap_or_default();

        PeerStore::new(addr_manager, ban_list)
    }

    #[cfg(target_family = "wasm")]
    pub fn load_from_config(config: &ckb_app_config::NetworkConfig) -> Self {
        let addr_manager = Ok((config.peer_store_fns.load_fn)())
            .and_then(|file| {
                AddrManager::load(std::io::BufReader::new(std::io::Cursor::new(file)))
                    .map_err(|err| error!("Failed to load AddrManager db, error: {:?}", err))
            })
            .unwrap_or_default();
        let ban_list = Ok((config.peer_store_ban_list_fns.load_fn)())
            .and_then(|file| {
                BanList::load(std::io::BufReader::new(std::io::Cursor::new(file)))
                    .map_err(|err| error!("Failed to load BanList db, error: {:?}", err))
            })
            .unwrap_or_default();
        PeerStore::new(addr_manager, ban_list)
    }

    /// Dump all info to disk
    pub fn dump_to_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        // create dir
        create_dir_all(&path)?;
        // dump file to a temporary sub-directory
        let tmp_dir = path.as_ref().join("tmp");
        create_dir_all(&tmp_dir)?;
        let tmp_addr_manager = tmp_dir.join(DEFAULT_ADDR_MANAGER_DB);
        let tmp_ban_list = tmp_dir.join(DEFAULT_BAN_LIST_DB);
        self.addr_manager().dump(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .append(false)
                .open(&tmp_addr_manager)?,
        )?;
        move_file(
            tmp_addr_manager,
            path.as_ref().join(DEFAULT_ADDR_MANAGER_DB),
        )?;
        self.ban_list().dump(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .append(false)
                .open(&tmp_ban_list)?,
        )?;
        move_file(tmp_ban_list, path.as_ref().join(DEFAULT_BAN_LIST_DB))?;
        Ok(())
    }

    #[cfg(target_family = "wasm")]
    pub fn dump_with_config(&self, config: &ckb_app_config::NetworkConfig) {
        self.addr_manager()
            .dump_with_fn(&config.peer_store_fns.dump_fn);
        self.ban_list()
            .dump_with_fn(&config.peer_store_ban_list_fns.dump_fn);
    }
}

/// This function use `copy` then `remove_file` as a fallback when `rename` failed,
/// this maybe happen when src and dst on different file systems.
fn move_file<P: AsRef<Path>>(src: P, dst: P) -> Result<(), Error> {
    if rename(&src, &dst).is_err() {
        copy(&src, &dst)?;
        remove_file(&src)?;
    }
    Ok(())
}
