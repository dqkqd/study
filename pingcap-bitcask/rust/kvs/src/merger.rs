use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};

use crate::{
    datafile::TimedLocation,
    directory::{readonly_datafile, Directory, FileId},
    kvs::{read_lock, write_lock, KeyLocations},
    KvOption, Result,
};

#[derive(Debug)]
pub struct MergeInfo {
    pub new_readonly_datafile_id: FileId,
    pub readonly_datafile_ids: Vec<FileId>,
    pub key_locations: KeyLocations,
}

fn get_merge_info(
    directory: &Arc<RwLock<Directory>>,
    key_locations: &Arc<RwLock<KeyLocations>>,
    options: KvOption,
) -> Result<Option<MergeInfo>> {
    let directory_reader = read_lock(directory)?;

    // No need to merge
    if directory_reader.readonly_datafiles.len() < options.num_readonly_datafiles {
        return Ok(None);
    }

    let mut new_datafile = directory_reader.next_active_datafile()?;
    let readonly_datafile_ids: BTreeSet<FileId> = directory_reader
        .readonly_datafiles
        .keys()
        .cloned()
        .collect();
    let path = directory_reader.path.clone();
    drop(directory_reader);

    let locations: Vec<TimedLocation> = {
        read_lock(key_locations)?
            .values()
            .cloned()
            .filter(|location| readonly_datafile_ids.contains(&location.loc.id))
            .collect()
    };

    let mut new_key_locations: BTreeMap<String, TimedLocation> = BTreeMap::new();
    for location in locations {
        let datafile = readonly_datafile(&path, &location.loc.id);
        let command = datafile.read(&location.loc)?;
        let new_location = new_datafile.location.timed_location(command.timestamp());
        new_datafile.write(&command)?;
        merge_location(&mut new_key_locations, command.key(), new_location);
    }

    Ok(Some(MergeInfo {
        new_readonly_datafile_id: new_datafile.id,
        readonly_datafile_ids: readonly_datafile_ids.into_iter().collect(),
        key_locations: new_key_locations,
    }))
}

fn merge(
    directory: &Arc<RwLock<Directory>>,
    key_locations: &Arc<RwLock<KeyLocations>>,
    merge_info: MergeInfo,
) -> Result<()> {
    {
        // transfer new key
        let mut key_locations = write_lock(key_locations)?;
        for (key, location) in merge_info.key_locations {
            merge_location(&mut key_locations, key, location)
        }
    }

    {
        let mut directory = write_lock(directory)?;

        // set the new readonly file id pointing to `new_datafile`
        let datafile = readonly_datafile(&directory.path, &merge_info.new_readonly_datafile_id);
        directory
            .readonly_datafiles
            .insert(merge_info.new_readonly_datafile_id, datafile);

        // remove old file ids
        for file_id in merge_info.readonly_datafile_ids {
            directory.remove_readonly_datafile(&file_id)?;
        }
    }

    Ok(())
}

pub(crate) fn background_merge(
    directory: &Arc<RwLock<Directory>>,
    key_locations: &Arc<RwLock<KeyLocations>>,
    options: KvOption,
) -> Result<()> {
    if let Some(merge_info) = get_merge_info(directory, key_locations, options)? {
        merge(directory, key_locations, merge_info)?;
    }
    Ok(())
}

pub(crate) fn merge_location(
    key_locations: &mut BTreeMap<String, TimedLocation>,
    key: String,
    location: TimedLocation,
) {
    key_locations
        .entry(key)
        .and_modify(|old_location| {
            if old_location.timestamp < location.timestamp {
                *old_location = location;
            }
        })
        .or_insert(location);
}
