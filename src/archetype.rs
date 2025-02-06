use crate::component::ComponentInfo;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub(crate) struct Archetype {
    bitfield: u128,
}

impl Archetype {
    #[inline(always)]
    pub fn new() -> Self {
        Archetype { bitfield: 0 }
    }

    #[inline(always)]
    pub fn set_id(&mut self, id: usize) {
        if id > 127 {
            todo!()
        }

        self.bitfield |= 1 << id;
    }

    #[inline(always)]
    pub fn set(&mut self, info: ComponentInfo) {
        self.set_id(*info.id() as usize);
    }

    #[inline(always)]
    pub fn unset_id(&mut self, id: usize) {
        if id > 127 {
            todo!()
        }

        self.bitfield &= !(1 << id);
    }

    #[inline(always)]
    pub fn unset(&mut self, info: ComponentInfo) {
        self.unset_id(*info.id() as usize);
    }

    #[inline(always)]
    pub fn contains_id(&self, id: usize) -> bool {
        self.bitfield & (1 << id) > 0
    }

    #[inline(always)]
    pub fn contains(&self, info: ComponentInfo) -> bool {
        self.contains_id(*info.id() as usize)
    }
}
