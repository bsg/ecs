extern crate codegen;
pub use codegen::Component;

mod archetype;
pub mod component;
mod store;
mod test;

use archetype::Archetype;
use component::{Component, ComponentId, ComponentInfo};
use store::Store;

use std::{cell::UnsafeCell, collections::HashMap, mem, ops::Deref};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Entity(usize);

impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

impl Component for Entity {
    fn info(&self) -> ComponentInfo {
        ComponentInfo::new(ComponentId(0), mem::size_of::<Entity>())
    }

    fn info_static() -> ComponentInfo
    where
        Self: Sized,
    {
        ComponentInfo::new(ComponentId(0), mem::size_of::<Entity>())
    }
}

trait QueryParam<'a, T: Component, A> {
    fn info() -> ComponentInfo;
    fn access(store: &'a Store, index: usize) -> A;
    fn match_archetype(archetype: &Archetype) -> bool;
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a T> for &'a T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn access(store: &'a Store, index: usize) -> &'a T {
        unsafe { store.read::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a mut T> for &'a mut T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn access(store: &'a Store, index: usize) -> &'a mut T {
        unsafe { store.read_mut::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Option<&'a T>> for Option<&'a T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn access(store: &'a Store, index: usize) -> Option<&'a T> {
        unsafe { store.try_read::<T>(index) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Option<&'a mut T>> for Option<&'a mut T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    fn access(store: &'a Store, index: usize) -> Option<&'a mut T> {
        unsafe { store.try_read_mut::<T>(index) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub trait System<'a, Params> {
    fn run(&mut self, world: &'a World);
}

macro_rules! impl_system {
    ($(($param:ident, $type:ident)),+) => {
        impl<'a, $($param,)+ $($type,)+ F> System<'a, ($($param,)+ $($type,)+)> for F
        where
            $($type: Component,)+
            $($param: QueryParam<'a, $type, $param>,)+
            F: FnMut($($param,)+),
        {
            fn run(&mut self, world: &'a World) {
                unsafe {
                    for (archetype, (store, store_len)) in world.stores.get().as_mut().unwrap().iter_mut() {
                        if $($param::match_archetype(archetype)) &&+ && true {
                            let mut lock = Archetype::new();
                            $(if *($param::info().id()) != 0 {lock.set($param::info());}) // FIXME don't hardcode the ignored id check
                            **world.lock.get().as_mut().unwrap() = Some(lock);

                            let mut item_idx = 0;
                            while item_idx < *store_len {
                                self(
                                    $($param::access(store, item_idx),)+
                                );
                                item_idx += 1;
                            }

                            *world.lock.get().as_mut().unwrap() = None;
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
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10),
    (A11, T11)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10),
    (A11, T11),
    (A12, T12)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10),
    (A11, T11),
    (A12, T12),
    (A13, T13)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10),
    (A11, T11),
    (A12, T12),
    (A13, T13),
    (A14, T14)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10),
    (A11, T11),
    (A12, T12),
    (A13, T13),
    (A14, T14),
    (A15, T15)
);
impl_system!(
    (A1, T1),
    (A2, T2),
    (A3, T3),
    (A4, T4),
    (A5, T5),
    (A6, T6),
    (A7, T7),
    (A8, T8),
    (A9, T9),
    (A10, T10),
    (A11, T11),
    (A12, T12),
    (A13, T13),
    (A14, T14),
    (A15, T15),
    (A16, T16)
);

pub struct World {
    // TODO group these into one UnsafeCell<Inner>
    entities: UnsafeCell<Vec<Option<(Archetype, usize)>>>,
    stores: UnsafeCell<HashMap<Archetype, (Store, usize)>>, // TODO move nrows into store
    lock: UnsafeCell<Option<Archetype>>,
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        World {
            entities: UnsafeCell::new(Vec::new()),
            stores: UnsafeCell::new(HashMap::new()),
            lock: UnsafeCell::new(None),
        }
    }

    pub fn spawn(&self, bundle: &[&dyn Component]) -> Entity {
        unsafe {
            let entity = Entity(self.entities.get().as_ref().unwrap().len());
            let mut archetype = Archetype::new();

            for component in bundle {
                archetype.set(component.info());
            }

            if let Some(lock) = self.lock.get().as_ref().unwrap() {
                if lock.is_subset_of(&archetype) {
                    panic!("")
                }
            }

            archetype.set(Entity::info_static());

            if !self.stores.get().as_ref().unwrap().contains_key(&archetype) {
                self.stores
                    .get()
                    .as_mut()
                    .unwrap()
                    .insert(archetype.clone(), (Store::new(), 0));
            }

            let (store, nrows) = self
                .stores
                .get()
                .as_mut()
                .unwrap()
                .get_mut(&archetype)
                .unwrap();
            store.write::<Entity>(*nrows, entity);
            for component in bundle {
                store.write_any(component.info(), *nrows, *component);
            }
            self.entities
                .get()
                .as_mut()
                .unwrap()
                .push(Some((archetype, *nrows)));
            (*nrows) += 1;

            entity
        }
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        unsafe {
            if let Some(Some((archetype, index))) =
                self.entities.get().as_ref().unwrap().get(entity.0)
            {
                Some(
                    self.stores
                        .get()
                        .as_ref()
                        .unwrap()
                        .get(archetype)
                        .unwrap()
                        .0
                        .read::<T>(*index),
                )
            } else {
                None
            }
        }
    }

    pub fn get_component_mut<T: Component + 'static>(&self, entity: Entity) -> Option<&mut T> {
        unsafe {
            if let Some(Some((archetype, index))) =
                self.entities.get().as_ref().unwrap().get(entity.0)
            {
                Some(
                    self.stores
                        .get()
                        .as_mut()
                        .unwrap()
                        .get_mut(archetype)
                        .unwrap()
                        .0
                        .read_mut::<T>(*index),
                )
            } else {
                None
            }
        }
    }

    pub fn run<'a, Params>(&'a self, mut f: impl System<'a, Params>) {
        f.run(self)
    }
}
