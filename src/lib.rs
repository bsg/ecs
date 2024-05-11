extern crate codegen;
pub use codegen::Component;

mod archetype;
pub mod component;
mod store;
mod test;

use archetype::Archetype;
use component::{Component, ComponentInfo};
use store::Store;

use std::{collections::HashMap, marker::PhantomData, ops::Deref};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Entity(usize);

impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

pub trait QueryParam<T: Component> {
    fn info() -> ComponentInfo;
    fn is_optional() -> bool;
}

impl<T: Component> QueryParam<T> for &T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn is_optional() -> bool {
        false
    }
}

impl<T: Component> QueryParam<T> for &mut T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn is_optional() -> bool {
        false
    }
}

impl<T: Component> QueryParam<T> for Option<&T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn is_optional() -> bool {
        true
    }
}

impl<T: Component> QueryParam<T> for Option<&mut T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn is_optional() -> bool {
        true
    }
}

#[allow(dead_code)]
pub struct Access<'a, T: Component> {
    store: &'a Store,
    index: usize,
    marker: PhantomData<T>,
}

pub trait System<'a, Params> {
    fn run(&mut self, world: &'a mut World);
}

macro_rules! impl_system {
    ($(($access:ident, $type:ident)),+) => {
        impl<'a, $($access,)+ $($type,)+ F> System<'a, ($($access,)+ $($type,)+)> for F
        where
            $($type: Component,)+
            $($access: QueryParam<$type> + From<Access<'a, $type>>,)+
            F: FnMut($($access,)+),
        {
            fn run(&mut self, world: &'a mut World) {
                for (archetype, (store, store_len)) in world.stores.iter_mut() {
                    if $(($access::is_optional() || archetype.contains($access::info()))) &&+ && true {
                        let mut item_idx = 0;
                        while item_idx < *store_len {
                            self(
                                $(Access::<$type> { store, index: item_idx, marker: PhantomData }.into(),)+
                            );
                            item_idx += 1;
                        }
                    }
                }
            }
        }
    }
}

// FIXME there is a better way to do this using a proc_macro but i'm too tired
impl_system!((A1, T1));
impl_system!((A1, T1), (A2, T2));
impl_system!((A1, T1), (A2, T2), (A3, T3));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10), (A11, T11));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10), (A11, T11), (A12, T12));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10), (A11, T11), (A12, T12), (A13, T13));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10), (A11, T11), (A12, T12), (A13, T13), (A14, T14));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10), (A11, T11), (A12, T12), (A13, T13), (A14, T14), (A15, T15));
impl_system!((A1, T1), (A2, T2), (A3, T3), (A4, T4), (A5, T5), (A6, T6), (A7, T7), (A8, T8), (A9, T9), (A10, T10), (A11, T11), (A12, T12), (A13, T13), (A14, T14), (A15, T15), (A16, T16));

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
            unsafe { store.write_any(component.info(), *nrows, *component) };
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

    pub fn run<'a, Params>(&'a mut self, mut f: impl System<'a, Params>) {
        f.run(self)
    }
}
