use crate::component::ComponentInfo;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Archetype {
    bitfield: u128,
}

impl Archetype {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Archetype { bitfield: 0 }
    }

    #[inline(always)]
    pub(crate) fn set_id(&mut self, id: usize) {
        if id > 127 {
            todo!()
        }

        self.bitfield |= 1 << id;
    }

    #[inline(always)]
    pub(crate) fn set(&mut self, info: ComponentInfo) {
        self.set_id(*info.id() as usize);
    }

    #[inline(always)]
    pub(crate) fn unset_id(&mut self, id: usize) {
        if id > 127 {
            todo!()
        }

        self.bitfield &= !(1 << id);
    }

    #[inline(always)]
    pub(crate) fn unset(&mut self, info: ComponentInfo) {
        self.unset_id(*info.id() as usize);
    }

    #[inline(always)]
    pub(crate) fn contains_id(&self, id: usize) -> bool {
        self.bitfield & (1 << id) > 0
    }

    #[inline(always)]
    pub fn contains(&self, info: ComponentInfo) -> bool {
        self.contains_id(*info.id() as usize)
    }

    pub fn subset_of(&self, other: Archetype) -> bool {
        (self.bitfield & other.bitfield) == self.bitfield
    }
}
