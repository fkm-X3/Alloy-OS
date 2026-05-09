// Vnode and operations for VFS (skeleton)

pub type VnodeId = u64;

pub enum VnodeType {
    File,
    Directory,
    CharDevice,
}

pub struct Vnode {
    pub id: VnodeId,
    pub vtype: VnodeType,
}

pub trait VnodeOps {
    fn read(&self, offset: usize, buf: &mut [u8]) -> isize;
    fn write(&self, offset: usize, buf: &[u8]) -> isize;
    fn getattr(&self) -> i32;
}

impl Vnode {
    pub fn new(id: VnodeId, vtype: VnodeType) -> Self {
        Vnode { id, vtype }
    }
}
