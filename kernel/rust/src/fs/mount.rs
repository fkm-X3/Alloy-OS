use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;

#[derive(Clone, Debug, PartialEq)]
pub enum FsType {
    TmpFs,
    Fat32,
}

pub struct MountPoint {
    pub fs_type: FsType,
    pub device_id: Option<usize>,
}

pub struct MountTable {
    mounts: BTreeMap<String, MountPoint>,
}

impl MountTable {
    pub fn new() -> Self {
        crate::println!("[Mount] MountTable::new start");
        let mut mounts = BTreeMap::new();
        let key = String::from("/");
        let val = MountPoint {
            fs_type: FsType::TmpFs,
            device_id: None,
        };
        crate::println!("[Mount] About to insert");
        mounts.insert(key, val);
        crate::println!("[Mount] Insert done");
        crate::println!("[Mount] Building struct");
        MountTable { mounts }
    }

    pub fn mount(
        &mut self,
        path: &str,
        fs_type: FsType,
        device_id: Option<usize>,
    ) -> Result<(), ()> {
        let norm = normalize_path(path);
        if self.mounts.contains_key(&norm) {
            return Err(());
        }
        self.mounts.insert(norm, MountPoint { fs_type, device_id });
        Ok(())
    }

    pub fn unmount(&mut self, path: &str) -> Result<(), ()> {
        let norm = normalize_path(path);
        if norm == "/" {
            return Err(());
        }
        self.mounts.remove(&norm).map(|_| ()).ok_or(())
    }

    pub fn resolve(&self, path: &str) -> (String, Option<&MountPoint>) {
        let norm = normalize_path(path);
        let mut best_match: Option<(usize, &str)> = None;

        for (mount_path, _mp) in &self.mounts {
            if norm == *mount_path || norm.starts_with(mount_path.as_str()) {
                let after = mount_path.len();
                if norm.len() == after || norm.as_bytes().get(after) == Some(&b'/') {
                    let prev = best_match.map(|(p, _)| p).unwrap_or(0);
                    if mount_path.len() > prev {
                        best_match = Some((mount_path.len(), mount_path.as_str()));
                    }
                }
            }
        }

        match best_match {
            Some((len, mp)) => {
                let relative = if norm.len() > len {
                    norm[len..].trim_start_matches('/')
                } else {
                    ""
                };
                (String::from(relative), self.mounts.get(mp))
            }
            None => (norm, None),
        }
    }
}

pub fn make_mount_table() -> Box<MountTable> {
    crate::println!("[Mount] make_mount_table start");
    let mut mounts = BTreeMap::new();
    let key = String::from("/");
    let val = MountPoint {
        fs_type: FsType::TmpFs,
        device_id: None,
    };
    mounts.insert(key, val);
    crate::println!("[Mount] make_mount_table insert done");
    let mt = Box::new(MountTable { mounts });
    crate::println!("[Mount] MountTable created");
    mt
}

fn normalize_path(path: &str) -> String {
    let mut out = String::new();
    let mut prev_slash = false;
    for b in path.as_bytes() {
        if *b == b'/' {
            if !prev_slash {
                out.push('/');
                prev_slash = true;
            }
        } else {
            out.push(*b as char);
            prev_slash = false;
        }
    }
    if out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    if out.is_empty() {
        out.push('/')
    }
    out
}

pub struct DeviceNode {
    pub name: String,
    pub device_id: usize,
}
