use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    datafile::{ActiveDatafile, ReadonlyDatafile},
    error::Result,
    KvError,
};

const DATAFILE_PREFIX: &str = "DATAFILE";
const DATAFILE_EXT: &str = "wal";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FileId(pub u64);

/// Struct that maintains all datafiles in database directory
#[derive(Debug)]
pub(crate) struct Directory {
    pub path: PathBuf,
    pub active_datafile: ActiveDatafile,
    pub readonly_datafiles: BTreeMap<FileId, ReadonlyDatafile>,
}

impl Directory {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Directory> {
        fs::create_dir_all(&path)?;

        let readonly_datafiles: BTreeMap<FileId, ReadonlyDatafile> = all_datafile_ids(&path)?
            .into_iter()
            .map(|file_id| (file_id, readonly_datafile(&path, &file_id)))
            .collect();

        let active_datafile = next_active_datafile(&path)?;

        Ok(Directory {
            path: path.as_ref().to_path_buf(),
            active_datafile,
            readonly_datafiles,
        })
    }

    pub fn rollover_active_datafile(&mut self) {
        if let Ok(active_datafile) = self.next_active_datafile() {
            self.readonly_datafiles.insert(
                self.active_datafile.id,
                self.readonly_datafile(&self.active_datafile.id),
            );

            self.active_datafile = active_datafile;
        }
    }

    pub fn next_active_datafile(&self) -> Result<ActiveDatafile> {
        next_active_datafile(&self.path)
    }

    pub fn readonly_datafile(&self, id: &FileId) -> ReadonlyDatafile {
        readonly_datafile(&self.path, id)
    }

    pub fn remove_readonly_datafile(&mut self, id: &FileId) -> Result<()> {
        if self.readonly_datafiles.remove(id).is_none() {
            return Err(KvError::FileIdDoesNotExist(id.0));
        }
        let path = datafile_path(&self.path, id);
        fs::remove_file(path)?;
        Ok(())
    }
}

fn datafile_path<P: AsRef<Path>>(path: P, id: &FileId) -> PathBuf {
    path.as_ref().join(format!(
        "{}_{:0>10}.{}",
        DATAFILE_PREFIX, id.0, DATAFILE_EXT
    ))
}

pub(crate) fn readonly_datafile<P: AsRef<Path>>(folder: P, id: &FileId) -> ReadonlyDatafile {
    ReadonlyDatafile::new(*id, datafile_path(folder, id))
}

fn next_free_file_id<P: AsRef<Path>>(folder: P) -> FileId {
    let mut id = FileId(0);
    loop {
        if !datafile_path(&folder, &id).exists() {
            break;
        }
        id.0 += 1
    }
    id
}

fn next_active_datafile<P: AsRef<Path>>(folder: P) -> Result<ActiveDatafile> {
    let file_id = next_free_file_id(&folder);
    let path = datafile_path(&folder, &file_id);
    ActiveDatafile::open(file_id, path)
}

fn all_datafile_ids<P: AsRef<Path>>(folder: P) -> Result<Vec<FileId>> {
    let pattern = format!(
        "{}/{}_*.{}",
        folder.as_ref().display(),
        DATAFILE_PREFIX,
        DATAFILE_EXT
    );

    let file_ids = glob::glob(&pattern)?
        .filter_map(|path| {
            path.ok().and_then(|path| {
                path.file_stem()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.split("_").last())
                    .and_then(|name| name.parse::<u64>().ok())
                    .map(FileId)
            })
        })
        .collect();

    Ok(file_ids)
}
