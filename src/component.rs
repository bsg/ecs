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
    fn metadata(&self) -> Metadata;
    fn metadata_static() -> Metadata
    where
        Self: Sized;
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Metadata {
    id: ComponentId,
    size: usize,
    name: &'static str,
}

impl Metadata {
    pub fn new(id: ComponentId, size: usize, name: &'static str) -> Self {
        Metadata { id, size, name }
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}
