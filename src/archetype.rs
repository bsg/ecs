use std::collections::BTreeSet;

use crate::component::{ComponentId, ComponentInfo};

// TODO implement as a bitfield
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Archetype {
    component_ids: BTreeSet<ComponentId>,
    size: usize,
}

impl Archetype {
    pub fn new() -> Self {
        Archetype {
            component_ids: BTreeSet::new(),
            size: 0,
        }
    }

    pub unsafe fn set(&mut self, info: ComponentInfo) -> bool {
        if self.component_ids.insert(info.id()) {
            self.size += info.size();
            true
        } else {
            false
        }
    }

    pub fn unset(&mut self, info: ComponentInfo) -> bool {
        if self.component_ids.remove(&info.id()) {
            self.size -= info.size();
            true
        } else {
            false
        }
    }

    pub fn contains(&self, info: ComponentInfo) -> bool {
        self.component_ids.contains(&info.id())
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
