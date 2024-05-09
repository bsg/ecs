extern crate codegen;
pub use codegen::Component;

pub mod component;
mod archetype;
mod store;
mod test;

use component::Component;
use archetype::Archetype;
use store::Store;

use std::{collections::HashMap, ops::Deref};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Entity(usize);

impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

pub trait System<Args> {
    fn run(&mut self, world: &mut World);
}

macro_rules! impl_system {
    ($($idt:ident),+) => {
        impl<$($idt,)+ F> System<($($idt,)+)> for F
        where
            $($idt: Component + 'static,)+
            F: FnMut($(&mut $idt,)+)
        {
            fn run(&mut self, world: &mut World) {
                for (archetype, (store, store_len)) in world.stores.iter_mut() {
                    if $(archetype.contains($idt::info_static()) &&)+ true {
                        let mut item_idx = 0;
                        while item_idx < *store_len {
                            unsafe {
                                self($(store.read_mut::<$idt>(item_idx),)+);
                            }
                            item_idx += 1;
                        }
                    }
                }
            }
        }
    };
}

impl_system!(T1);
impl_system!(T1, T2);
impl_system!(T1, T2, T3);
impl_system!(T1, T2, T3, T4);
impl_system!(T1, T2, T3, T4, T5);
impl_system!(T1, T2, T3, T4, T5, T6);
impl_system!(T1, T2, T3, T4, T5, T6, T7);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

pub struct World {
    entities: Vec<Option<(Archetype, Entity)>>,
    stores: HashMap<Archetype, (Store, usize)>,
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            stores: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, bundle: &[&dyn Component]) -> Entity {
        let mut archetype = Archetype::new();

        for component in bundle {
            unsafe { archetype.set(component.info()) };
        }

        if !self.stores.contains_key(&archetype) {
            self.stores.insert(archetype.clone(), (Store::new(), 0));
        }

        let (store, nrows) = self.stores.get_mut(&archetype).unwrap();
        for component in bundle {
            unsafe {
                store.write_any(
                    component.info().id(),
                    component.info().size(),
                    *nrows,
                    *component,
                )
            };
        }
        self.entities.push(Some((archetype, Entity(*nrows))));
        (*nrows) += 1;

        Entity(self.entities.len() - 1)
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        if let Some(Some((archetype, index))) = self.entities.get(entity.0) {
            Some(unsafe { self.stores.get(archetype).unwrap().0.read::<T>(**index) })
        } else {
            None
        }
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(Some((archetype, index))) = self.entities.get(entity.0) {
            Some(unsafe {
                self.stores
                    .get_mut(archetype)
                    .unwrap()
                    .0
                    .read_mut::<T>(**index)
            })
        } else {
            None
        }
    }

    pub fn run<Args>(&mut self, mut f: impl System<Args>) {
        f.run(self)
    }
}
