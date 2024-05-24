use crate::component::ComponentInfo;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Archetype {
    bitfield: u128,
    size: usize,
}

impl Archetype {
    pub fn new() -> Self {
        Archetype {
            bitfield: 0,
            size: 0,
        }
    }

    pub fn set(&mut self, info: ComponentInfo) {
        if *info.id() > 127 {
            todo!()
        }

        self.bitfield |= 1 << *info.id();
    }

    pub fn unset(&mut self, info: ComponentInfo) {
        if *info.id() > 127 {
            todo!()
        }

        self.bitfield ^= 1 << *info.id();
    }
    
    pub fn contains(&self, info: ComponentInfo) -> bool {
        self.bitfield & (1 << *info.id()) > 0
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
