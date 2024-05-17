extern crate codegen;
pub use codegen::Component;

mod archetype;
pub mod component;
mod store;
mod test;

use archetype::Archetype;
use component::{Component, ComponentId, ComponentInfo};
use store::{ComponentList, Store};

use std::{
    cell::UnsafeCell,
    collections::{BTreeSet, HashMap},
    mem,
    ops::Deref,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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
    fn access(list: Option<&'a ComponentList>, index: usize) -> A;
    fn match_archetype(archetype: &Archetype) -> bool;
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a T> for &'a T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(store: Option<&'a ComponentList>, index: usize) -> &'a T {
        unsafe { store.unwrap_unchecked().read::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a mut T> for &'a mut T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(store: Option<&'a ComponentList>, index: usize) -> &'a mut T {
        unsafe { store.unwrap_unchecked().read_mut::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Option<&'a T>> for Option<&'a T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(store: Option<&'a ComponentList>, index: usize) -> Option<&'a T> {
        unsafe { store.map(|list| list.read::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Option<&'a mut T>> for Option<&'a mut T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(store: Option<&'a ComponentList>, index: usize) -> Option<&'a mut T> {
        unsafe { store.map(|list| list.read_mut::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub trait System<'a, Params> {
    fn run(&mut self, world: &'a World);
}

macro_rules! impl_system {
    ($(($param:ident, $type:ident, $list:ident)),+) => {
        impl<'a, $($param,)+ $($type,)+ F> System<'a, ($($param,)+ $($type,)+)> for F
        where
            $($type: Component + 'static,)+
            $($param: QueryParam<'a, $type, $param>,)+
            F: FnMut($($param,)+),
        {
            fn run(&mut self, world: &'a World) {
                unsafe {
                    for (archetype, store) in world.stores.get().as_mut().unwrap().iter_mut() {
                        if $($param::match_archetype(archetype)) &&+ && true {
                            let mut item_idx = 0;
                            let len = store.len();
                            let entities = store.get_component_list::<Entity>().unwrap();
                            $(let $list = store.get_component_list::<$type>();)+
                            while item_idx < len {
                                if entities.read::<Entity>(item_idx).0 != 0 {
                                    self(
                                        $($param::access($list, item_idx),)+
                                    );
                                }
                                item_idx += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

// FIXME there is a better way to do this using a proc_macro but i'm too tired
impl_system!((A1, T1, r1));
impl_system!((A1, T1, r1), (A2, T2, r2));
impl_system!((A1, T1, r1), (A2, T2, r2), (A3, T3, r3));
impl_system!((A1, T1, r1), (A2, T2, r2), (A3, T3, r3), (A4, T4, r4));
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10),
    (A11, T11, r11)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10),
    (A11, T11, r11),
    (A12, T12, r12)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10),
    (A11, T11, r11),
    (A12, T12, r12),
    (A13, T13, r13)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10),
    (A11, T11, r11),
    (A12, T12, r12),
    (A13, T13, r13),
    (A14, T14, r14)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10),
    (A11, T11, r11),
    (A12, T12, r12),
    (A13, T13, r13),
    (A14, T14, r14),
    (A15, T15, r15)
);
impl_system!(
    (A1, T1, r1),
    (A2, T2, r2),
    (A3, T3, r3),
    (A4, T4, r4),
    (A5, T5, r5),
    (A6, T6, r6),
    (A7, T7, r7),
    (A8, T8, r8),
    (A9, T9, r9),
    (A10, T10, r10),
    (A11, T11, r11),
    (A12, T12, r12),
    (A13, T13, r13),
    (A14, T14, r14),
    (A15, T15, r15),
    (A16, T16, r16)
);

pub struct World {
    // TODO group these into one UnsafeCell<Inner>
    entities: UnsafeCell<Vec<Option<(Archetype, usize)>>>,
    stores: UnsafeCell<HashMap<Archetype, Store>>,
    free_entities: UnsafeCell<BTreeSet<Entity>>,
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        World {
            // Entity(0) is used to mark deleted columns
            entities: UnsafeCell::new(vec![None]), 
            stores: UnsafeCell::new(HashMap::new()),
            free_entities: UnsafeCell::new(BTreeSet::new()),
        }
    }

    pub fn spawn(&self, bundle: &[&dyn Component]) -> Entity {
        unsafe {
            let entity = match self.free_entities.get().as_mut().unwrap().pop_first() {
                Some(e) => e,
                None => Entity(self.entities.get().as_ref().unwrap().len()),
            };

            let mut archetype = Archetype::new();

            for component in bundle {
                archetype.set(component.info());
            }

            archetype.set(Entity::info_static());

            if !self.stores.get().as_ref().unwrap().contains_key(&archetype) {
                self.stores
                    .get()
                    .as_mut()
                    .unwrap()
                    .insert(archetype.clone(), Store::new());
            }

            let store = self
                .stores
                .get()
                .as_mut()
                .unwrap()
                .get_mut(&archetype)
                .unwrap();
            let index = store.reserve_index();
            store.write::<Entity>(index, entity);
            for component in bundle {
                store.write_any(component.info(), index, *component);
            }
            match self.entities.get().as_mut().unwrap().get_mut(*entity) {
                Some(p) => *p = Some((archetype, index)),
                None => self
                    .entities
                    .get()
                    .as_mut()
                    .unwrap()
                    .push(Some((archetype, index))),
            }

            entity
        }
    }

    pub fn despawn(&self, entity: Entity) {
        unsafe {
            if entity == Entity(0) {
                return;
            }

            if let Some(Some((archetype, index))) =
                self.entities.get().as_mut().unwrap().get(*entity)
            {
                let store = self
                    .stores
                    .get()
                    .as_mut()
                    .unwrap()
                    .get_mut(archetype)
                    .unwrap();

                *store.read_mut::<Entity>(*index) = Entity(0);
                store.free_index(*index);

                *self
                    .entities
                    .get()
                    .as_mut()
                    .unwrap()
                    .get_mut(*entity)
                    .unwrap() = None;

                self.free_entities.get().as_mut().unwrap().insert(entity);
            }
        }
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        unsafe {
            if let Some(Some((archetype, index))) =
                self.entities.get().as_ref().unwrap().get(*entity)
            {
                self.stores
                    .get()
                    .as_ref()
                    .unwrap()
                    .get(archetype)
                    .unwrap()
                    .try_read::<T>(*index)
            } else {
                None
            }
        }
    }

    pub fn get_component_mut<T: Component + 'static>(&self, entity: Entity) -> Option<&mut T> {
        unsafe {
            if let Some(Some((archetype, index))) =
                self.entities.get().as_ref().unwrap().get(*entity)
            {
                self.stores
                    .get()
                    .as_mut()
                    .unwrap()
                    .get_mut(archetype)
                    .unwrap()
                    .try_read_mut::<T>(*index)
            } else {
                None
            }
        }
    }

    pub fn run<'a, Params>(&'a self, mut f: impl System<'a, Params>) {
        f.run(self)
    }
}
