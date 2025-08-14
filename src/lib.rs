extern crate codegen;
pub use codegen::Component;

mod archetype;
pub mod component;
mod store;
mod test;

use archetype::Archetype;
use component::{Component, ComponentId, ComponentInfo};
use serde::Deserialize;
use serde::Serialize;
use store::{ComponentList, Store};

use core::panic;
use std::mem::MaybeUninit;
use std::{
    cell::UnsafeCell,
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
    mem,
    ops::Deref,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Entity(pub u32);

impl Deref for Entity {
    type Target = u32;

    fn deref(&self) -> &u32 {
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

trait QueryParam<'a, T, A, C: Ctx> {
    fn access(world: &'a World<C>, list: Option<&'a ComponentList>, index: usize) -> A;
    fn match_archetype(archetype: &Archetype) -> bool;
}

impl<'a, T: Component + 'static, C: Ctx> QueryParam<'a, T, &'a T, C> for &'a T {
    #[inline(always)]
    fn access(_: &World<C>, store: Option<&'a ComponentList>, index: usize) -> &'a T {
        unsafe { store.unwrap_unchecked().read::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

impl<'a, T: Component + 'static, C: Ctx> QueryParam<'a, T, &'a mut T, C> for &'a mut T {
    #[inline(always)]
    fn access(_: &World<C>, store: Option<&'a ComponentList>, index: usize) -> &'a mut T {
        unsafe { store.unwrap_unchecked().read_mut::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

impl<'a, T: Component + 'static, C: Ctx> QueryParam<'a, T, Option<&'a T>, C> for Option<&'a T> {
    #[inline(always)]
    fn access(_: &World<C>, store: Option<&'a ComponentList>, index: usize) -> Option<&'a T> {
        unsafe { store.map(|list| list.read::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

impl<'a, T: Component + 'static, C: Ctx> QueryParam<'a, T, Option<&'a mut T>, C>
    for Option<&'a mut T>
{
    #[inline(always)]
    fn access(_: &World<C>, store: Option<&'a ComponentList>, index: usize) -> Option<&'a mut T> {
        unsafe { store.map(|list| list.read_mut::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub struct With<T: Component> {
    marker: PhantomData<T>,
}

impl<'a, T: Component + 'static, C: Ctx> QueryParam<'a, T, With<T>, C> for With<T> {
    #[inline(always)]
    fn access(_: &'a World<C>, _: Option<&'a ComponentList>, _: usize) -> With<T> {
        With {
            marker: PhantomData,
        }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

pub struct Without<T: Component> {
    marker: PhantomData<T>,
}

impl<'a, T: Component + 'static, C: Ctx> QueryParam<'a, T, Without<T>, C> for Without<T> {
    #[inline(always)]
    fn access(_: &'a World<C>, _: Option<&'a ComponentList>, _: usize) -> Without<T> {
        Without {
            marker: PhantomData,
        }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        !archetype.contains(T::info_static())
    }
}

pub trait System<'a, Params, C: Ctx> {
    fn run(&mut self, world: &'a World<C>);
}

macro_rules! impl_system {
    ($(($param:ident, $t:ident, $list:ident)),+) => {
        impl<'a, $($param,)+ $($t,)+ F, C: Ctx> System<'a, ($($param,)+ $($t,)+), C> for F
        where
            $($t: Component + 'static,)+
            $($param: QueryParam<'a, $t, $param, C>,)+
            F: FnMut($($param,)+),
        {
            fn run(&mut self, world: &'a World<C>) {
                unsafe {
                    for (archetype, store) in world.stores_mut().iter_mut() {
                        if $($param::match_archetype(archetype)) &&+ && true {
                            let mut item_idx = 0;
                            let len = store.len();
                            let entities = store.get_component_list::<Entity>().unwrap_unchecked();
                            $(let $list = store.get_component_list::<$t>();)+
                            while item_idx < len {
                                if entities.read::<Entity>(item_idx).0 != 0 {
                                    self(
                                        $($param::access(world, $list, item_idx),)+
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

pub trait Ctx {}

struct WorldInner<C: Ctx> {
    entities: Vec<Option<(Archetype, usize)>>,
    stores: HashMap<Archetype, Store>,
    free_entities: BTreeSet<Entity>,
    ctx: MaybeUninit<C>, // TODO  drop
}

pub struct World<C: Ctx> {
    inner: UnsafeCell<WorldInner<C>>,
}

unsafe impl<C: Ctx> Send for World<C> {}
unsafe impl<C: Ctx> Sync for World<C> {}

#[allow(dead_code)]
impl<C: Ctx> World<C> {
    pub fn new() -> Self {
        World {
            inner: UnsafeCell::new(WorldInner {
                // Entity(0) is used to mark deleted columns
                entities: vec![None],
                stores: HashMap::new(),
                free_entities: BTreeSet::new(),
                ctx: MaybeUninit::<C>::uninit(),
            }),
        }
    }

    pub fn init_ctx(&self, ctx: C) {
        unsafe { self.inner.get().as_mut().unwrap_unchecked().ctx = MaybeUninit::new(ctx) }
    }

    fn entities(&self) -> &Vec<Option<(Archetype, usize)>> {
        unsafe { &self.inner.get().as_ref().unwrap_unchecked().entities }
    }

    #[allow(clippy::mut_from_ref)]
    fn entities_mut(&self) -> &mut Vec<Option<(Archetype, usize)>> {
        unsafe { &mut self.inner.get().as_mut().unwrap_unchecked().entities }
    }

    fn stores(&self) -> &HashMap<Archetype, Store> {
        unsafe { &self.inner.get().as_ref().unwrap_unchecked().stores }
    }

    #[allow(clippy::mut_from_ref)]
    fn stores_mut(&self) -> &mut HashMap<Archetype, Store> {
        unsafe { &mut self.inner.get().as_mut().unwrap_unchecked().stores }
    }

    fn free_entities(&self) -> &BTreeSet<Entity> {
        unsafe { &self.inner.get().as_ref().unwrap_unchecked().free_entities }
    }

    #[allow(clippy::mut_from_ref)]
    fn free_entities_mut(&self) -> &mut BTreeSet<Entity> {
        unsafe { &mut self.inner.get().as_mut().unwrap_unchecked().free_entities }
    }

    pub fn spawn(&self, bundle: &[&(dyn Component + Send)]) -> Entity {
        let entity = self
            .free_entities_mut()
            .pop_first()
            .unwrap_or(Entity(self.entities().len() as u32));

        let mut archetype = Archetype::new();

        for item in bundle {
            archetype.set(item.info());
        }

        archetype.set(Entity::info_static());

        if !self.stores().contains_key(&archetype) {
            self.stores_mut().insert(archetype, Store::new());
        }

        let store = unsafe { self.stores_mut().get_mut(&archetype).unwrap_unchecked() };
        let index = store.reserve_index();
        unsafe { store.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { store.write_any(item.info(), index, *item) };
        }
        match self.entities_mut().get_mut(*entity as usize) {
            Some(p) => *p = Some((archetype, index)),
            None => self.entities_mut().push(Some((archetype, index))),
        }

        entity
    }

    // TODO tests
    pub fn insert(&self, entity: Entity, bundle: &[&(dyn Component + Send)]) -> Entity {
        let mut archetype = Archetype::new();

        for item in bundle {
            archetype.set(item.info());
        }

        archetype.set(Entity::info_static());

        if !self.stores().contains_key(&archetype) {
            self.stores_mut().insert(archetype, Store::new());
        }

        let store = unsafe { self.stores_mut().get_mut(&archetype).unwrap_unchecked() };
        let index = store.reserve_index();
        unsafe { store.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { store.write_any(item.info(), index, *item) };
        }

        if *entity as usize >= self.entities().len() {
            self.entities_mut().resize(*entity as usize + 1, None);
        }
        self.entities_mut().as_mut_slice()[*entity as usize] = Some((archetype, index));
        self.free_entities_mut().remove(&entity);

        entity
    }

    /// # Safety
    /// This is marked 'unsafe' because it's likely that component specific teardown
    /// will need to be implemented by the user
    pub unsafe fn despawn(&self, entity: Entity) {
        if entity == Entity(0) {
            return;
        }

        if let Some(Some((archetype, index))) = self.entities().get(*entity as usize) {
            let store = self.stores_mut().get_mut(archetype).unwrap_unchecked();

            *store.read_mut::<Entity>(*index) = Entity(0);
            store.free_index(*index);

            *self
                .entities_mut()
                .get_mut(*entity as usize)
                .unwrap_unchecked() = None;

            self.free_entities_mut().insert(entity);
        }
    }

    pub fn has_component<T: Component + 'static>(&self, entity: Entity) -> bool {
        if let Some(Some((archetype, _))) = self.entities().get(*entity as usize) {
            unsafe {
                self.stores()
                    .get(archetype)
                    .unwrap_unchecked()
                    .has_component::<T>()
            }
        } else {
            false
        }
    }

    pub fn component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        unsafe {
            if let Some(Some((archetype, index))) = self.entities().get(*entity as usize) {
                self.stores()
                    .get(archetype)
                    .unwrap_unchecked()
                    .try_read::<T>(*index)
            } else {
                None
            }
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn component_mut<T: Component + 'static>(&self, entity: Entity) -> Option<&mut T> {
        unsafe {
            if let Some(Some((archetype, index))) = self.entities().get(*entity as usize) {
                self.stores_mut()
                    .get_mut(archetype)
                    .unwrap_unchecked()
                    .try_read_mut::<T>(*index)
            } else {
                None
            }
        }
    }

    pub fn add_component<T: Component + 'static>(
        &self,
        entity: Entity,
        component: T,
    ) -> Result<(), ()> {
        if let Some(Some((archetype, index))) = self.entities_mut().get_mut(*entity as usize) {
            let mut new_archetype = *archetype;
            new_archetype.set(component.info());

            if *archetype == new_archetype {
                return Result::Err(());
            }

            if !self.stores().contains_key(&new_archetype) {
                self.stores_mut().insert(new_archetype, Store::new());

                let store = unsafe { self.stores_mut().get_mut(archetype).unwrap_unchecked() };
                let new_store =
                    unsafe { self.stores_mut().get_mut(&new_archetype).unwrap_unchecked() };
                for id in 0..128 {
                    unsafe {
                        if let Some(list) = store.get_component_list_by_id(ComponentId(id as u32)) {
                            if new_archetype.contains_id(id) {
                                new_store.add_component_list_by_id(
                                    ComponentId(id as u32),
                                    list.get_component_size(),
                                )
                            }
                        }
                    };
                }
            }

            let store = unsafe { self.stores_mut().get_mut(archetype).unwrap_unchecked() };
            let new_store = unsafe { self.stores_mut().get_mut(&new_archetype).unwrap_unchecked() };
            let new_index = new_store.reserve_index();

            for id in 0..128 {
                if archetype.contains_id(id) {
                    unsafe {
                        ComponentList::copy_item_from_list(
                            store
                                .get_component_list_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            new_store
                                .get_component_list_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            *index,
                            new_index,
                        );
                    }
                }
            }

            unsafe { store.write::<Entity>(*index, Entity(0)) };
            store.free_index(*index);

            unsafe { new_store.write(new_index, component) };

            *archetype = new_archetype;
            *index = new_index;

            return Result::Ok(());
        }

        Result::Err(())
    }

    pub fn remove_component<T: Component + 'static>(&self, entity: Entity) -> Result<(), ()> {
        if let Some(Some((archetype, index))) = self.entities_mut().get_mut(*entity as usize) {
            let mut new_archetype = *archetype;
            new_archetype.unset(T::info_static());

            if *archetype == new_archetype {
                return Result::Err(());
            }

            if !self.stores().contains_key(&new_archetype) {
                if self
                    .stores_mut()
                    .insert(new_archetype, Store::new())
                    .is_some()
                {
                    panic!("")
                }

                let store = unsafe { self.stores_mut().get_mut(archetype).unwrap_unchecked() };
                let new_store =
                    unsafe { self.stores_mut().get_mut(&new_archetype).unwrap_unchecked() };
                for id in 0..128usize {
                    unsafe {
                        if let Some(list) = store.get_component_list_by_id(ComponentId(id as u32)) {
                            if new_archetype.contains_id(id) {
                                new_store.add_component_list_by_id(
                                    ComponentId(id as u32),
                                    list.get_component_size(),
                                )
                            }
                        }
                    };
                }
            }

            let store = unsafe { self.stores_mut().get_mut(archetype).unwrap_unchecked() };
            let new_store = unsafe { self.stores_mut().get_mut(&new_archetype).unwrap_unchecked() };
            let new_index = new_store.reserve_index();

            for id in 0..128usize {
                if new_archetype.contains_id(id) {
                    unsafe {
                        ComponentList::copy_item_from_list(
                            store
                                .get_component_list_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            new_store
                                .get_component_list_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            *index,
                            new_index,
                        );
                    }
                }
            }

            unsafe { store.write::<Entity>(*index, Entity(0)) };
            store.free_index(*index);

            *archetype = new_archetype;
            *index = new_index;

            return Result::Ok(());
        }

        Result::Err(())
    }

    #[allow(clippy::mut_from_ref)]
    pub fn ctx(&self) -> &mut C {
        unsafe {
            self.inner
                .get()
                .as_mut()
                .unwrap_unchecked()
                .ctx
                .assume_init_mut()
        }
    }

    pub fn run<'a, Params>(&'a self, mut f: impl System<'a, Params, C>) {
        f.run(self)
    }

    // TODO rename
    pub fn num_entities_upper_bound(&self) -> u32 {
        self.entities().len() as u32
    }
}

impl<C: Ctx> Default for World<C> {
    fn default() -> Self {
        Self::new()
    }
}
