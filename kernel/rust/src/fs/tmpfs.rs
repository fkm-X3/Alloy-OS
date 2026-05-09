// Minimal tmpfs skeleton - in-memory filesystem placeholder

use crate::fs::vnode::{Vnode, VnodeType};

pub struct TmpFs {
    // TODO: Use a proper map from path -> Vnode + data
}

impl TmpFs {
    pub fn new() -> Self {
        TmpFs {}
    }

    pub fn mount(&self) {
        // TODO
    }
}

pub fn init_tmpfs() {
    // Initialize root tmpfs for now
}
