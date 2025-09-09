use std::ops::Deref;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ComponentId(pub u32);

impl Deref for ComponentId {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

pub trait Component {
    fn info(&self) -> ComponentInfo;
    fn info_static() -> ComponentInfo
    where
        Self: Sized;
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct ComponentInfo {
    id: ComponentId,
    size: u32,
    name: &'static str,
}

impl ComponentInfo {
    pub fn new(id: ComponentId, size: usize, name: &'static str) -> Self {
        ComponentInfo {
            id,
            size: u32::try_from(size).expect("Component size too large"),
            name,
        }
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }
}
