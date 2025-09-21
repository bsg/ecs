pub use ecs_codegen::component;

mod archetype;
pub mod component;
mod store;
mod test;

use archetype::Archetype;
use component::{ComponentId, ComponentInfo};
use serde::Deserialize;
use serde::Serialize;
use store::{ComponentList, Store};

use core::panic;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicUsize;
use std::{
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
    mem,
    ops::Deref,
};

use crate::component::Component;

#[derive(Clone, Copy)]
pub struct ArchetypeBuilder(Archetype);
impl ArchetypeBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ArchetypeBuilder {
        let mut archetype = Archetype::new();
        archetype.set(Entity::info_static());
        ArchetypeBuilder(archetype)
    }

    pub fn set<T: Component>(mut self) -> ArchetypeBuilder {
        self.0.set(T::info_static());
        self
    }

    pub fn set_from_info(&mut self, info: ComponentInfo) -> &mut ArchetypeBuilder {
        self.0.set(info);
        self
    }

    pub fn build(self) -> Archetype {
        self.0
    }
}

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
        ComponentInfo::new(ComponentId(0), mem::size_of::<Entity>(), "Entity")
    }

    fn info_static() -> ComponentInfo
    where
        Self: Sized,
    {
        ComponentInfo::new(ComponentId(0), mem::size_of::<Entity>(), "Entity")
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
                    for (archetype, store) in world.inner().stores.iter_mut() {
                        if $($param::match_archetype(archetype)) &&+ && true {
                            let len = store.len();
                            let entities = store.get_component_list::<Entity>().unwrap_unchecked();
                            $(let $list = store.get_component_list::<$t>();)+
                            for item_idx in 0..len {
                                if entities.read::<Entity>(item_idx).0 != 0 {
                                    self(
                                        $($param::access(world, $list, item_idx),)+
                                    );
                                }
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

enum Cmd {
    AddComponent((Entity, ComponentInfo, Box<dyn Component>)),
    RemoveComponent((Entity, ComponentInfo)),
}

struct WorldInner<C: Ctx> {
    entities: Vec<Option<(Archetype, usize)>>,
    stores: HashMap<Archetype, Store>,
    free_entities: BTreeSet<Entity>,
    ctx: MaybeUninit<C>, // TODO drop
    cmd_queue: Vec<Cmd>,
    num_systems_running: AtomicUsize,
}

pub struct World<C: Ctx> {
    inner: *mut WorldInner<C>,
}

unsafe impl<C: Ctx> Send for World<C> {}
unsafe impl<C: Ctx> Sync for World<C> {}

#[allow(dead_code)]
impl<C: Ctx> World<C> {
    pub fn new() -> Self {
        World {
            inner: Box::into_raw(Box::new(WorldInner {
                // Entity(0) is used to mark deleted columns
                entities: vec![None],
                stores: HashMap::new(),
                free_entities: BTreeSet::new(),
                ctx: MaybeUninit::<C>::uninit(),
                cmd_queue: Vec::default(),
                num_systems_running: AtomicUsize::new(0),
            })),
        }
    }

    pub unsafe fn from_raw(ptr: *mut u8) -> Self {
        World { inner: ptr.cast() }
    }

    pub unsafe fn as_raw(&self) -> *mut u8 {
        self.inner.cast()
    }

    pub unsafe fn set_inner_from_raw(&mut self, ptr: *mut u8) {
        self.inner = ptr.cast();
    }

    pub fn set_ctx(&self, ctx: C) {
        unsafe { (&mut *self.inner).ctx = MaybeUninit::new(ctx) }
    }

    #[allow(clippy::mut_from_ref)]
    fn inner(&self) -> &mut WorldInner<C> {
        unsafe { &mut *self.inner }
    }

    pub fn spawn(&self, bundle: &[&dyn Component]) -> Entity {
        let entity = self
            .inner()
            .free_entities
            .pop_first()
            .unwrap_or(Entity(self.inner().entities.len() as u32));

        let mut archetype = Archetype::new();

        for item in bundle {
            archetype.set(item.info());
        }

        archetype.set(Entity::info_static());

        self.inner()
            .stores
            .entry(archetype)
            .or_insert_with(Store::new);

        let store = unsafe { self.inner().stores.get_mut(&archetype).unwrap_unchecked() };
        let index = store.reserve_index();
        unsafe { store.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { store.write_any(item.info(), index, *item) };
        }
        match self.inner().entities.get_mut(*entity as usize) {
            Some(p) => *p = Some((archetype, index)),
            None => self.inner().entities.push(Some((archetype, index))),
        }

        entity
    }

    // FIXME this shares a ton of code with spawn()
    pub fn spawn_from_slice_of_boxes(&self, bundle: &[Box<dyn Component>]) -> Entity {
        let entity = self
            .inner()
            .free_entities
            .pop_first()
            .unwrap_or(Entity(self.inner().entities.len() as u32));

        let mut archetype = Archetype::new();

        for item in bundle {
            archetype.set(item.info());
        }

        archetype.set(Entity::info_static());

        self.inner()
            .stores
            .entry(archetype)
            .or_insert_with(Store::new);

        let store = unsafe { self.inner().stores.get_mut(&archetype).unwrap_unchecked() };
        let index = store.reserve_index();
        unsafe { store.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { store.write_any(item.info(), index, &**item) };
        }
        match self.inner().entities.get_mut(*entity as usize) {
            Some(p) => *p = Some((archetype, index)),
            None => self.inner().entities.push(Some((archetype, index))),
        }

        entity
    }

    // TODO tests
    // TODO this is almost identical to spawn(). dedup
    pub fn insert(&self, entity: Entity, bundle: &[&dyn Component]) -> Entity {
        let mut archetype = Archetype::new();

        for item in bundle {
            archetype.set(item.info());
        }

        archetype.set(Entity::info_static());

        self.inner()
            .stores
            .entry(archetype)
            .or_insert_with(Store::new);

        let store = unsafe { self.inner().stores.get_mut(&archetype).unwrap_unchecked() };
        let index = store.reserve_index();
        unsafe { store.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { store.write_any(item.info(), index, *item) };
        }

        if *entity as usize >= self.inner().entities.len() {
            self.inner().entities.resize(*entity as usize + 1, None);
            // TODO add the slots in the gap to the free list
        }
        self.inner().entities.as_mut_slice()[*entity as usize] = Some((archetype, index));
        self.inner().free_entities.remove(&entity);

        entity
    }

    /// # Safety
    /// This is marked 'unsafe' because it's likely that component specific teardown
    /// will need to be implemented by the user
    pub unsafe fn despawn(&self, entity: Entity) {
        if entity == Entity(0) {
            return;
        }

        if let Some(Some((archetype, index))) = self.inner().entities.get(*entity as usize) {
            let store = self.inner().stores.get_mut(archetype).unwrap_unchecked();

            *store.read_mut::<Entity>(*index) = Entity(0);
            store.free_index(*index);

            *self
                .inner()
                .entities
                .get_mut(*entity as usize)
                .unwrap_unchecked() = None;

            self.inner().free_entities.insert(entity);
        }
    }

    pub fn has_component<T: Component + 'static>(&self, entity: Entity) -> bool {
        if let Some(Some((archetype, _))) = self.inner().entities.get(*entity as usize) {
            unsafe {
                self.inner()
                    .stores
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
            if let Some(Some((archetype, index))) = self.inner().entities.get(*entity as usize) {
                self.inner()
                    .stores
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
            if let Some(Some((archetype, index))) = self.inner().entities.get(*entity as usize) {
                self.inner()
                    .stores
                    .get_mut(archetype)
                    .unwrap_unchecked()
                    .try_read_mut::<T>(*index)
            } else {
                None
            }
        }
    }

    pub fn add_component<T: Component + 'static>(&self, entity: Entity, component: T) {
        if self
            .inner()
            .num_systems_running
            .load(std::sync::atomic::Ordering::Relaxed)
            == 0
        {
            let _ = self._add_component(entity, T::info_static(), &component);
        } else {
            self.inner().cmd_queue.push(Cmd::AddComponent((
                entity,
                T::info_static(),
                Box::new(component),
            )));
        }
    }

    fn _add_component(
        &self,
        entity: Entity,
        component_info: ComponentInfo,
        component: &dyn Component,
    ) -> Result<(), ()> {
        if let Some(Some((archetype, index))) = self.inner().entities.get_mut(*entity as usize) {
            let mut new_archetype = *archetype;
            new_archetype.set(component.info());

            if *archetype == new_archetype {
                return Result::Err(());
            }

            if let std::collections::hash_map::Entry::Vacant(e) =
                self.inner().stores.entry(new_archetype)
            {
                e.insert(Store::new());

                let store = unsafe { self.inner().stores.get_mut(archetype).unwrap_unchecked() };
                let new_store = unsafe {
                    self.inner()
                        .stores
                        .get_mut(&new_archetype)
                        .unwrap_unchecked()
                };
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

            let store = unsafe { self.inner().stores.get_mut(archetype).unwrap_unchecked() };
            let new_store = unsafe {
                self.inner()
                    .stores
                    .get_mut(&new_archetype)
                    .unwrap_unchecked()
            };
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

            unsafe { new_store.write_any(component_info, new_index, component) };

            *archetype = new_archetype;
            *index = new_index;

            return Result::Ok(());
        }

        Result::Err(())
    }

    pub fn remove_component<T: Component + 'static>(&self, entity: Entity) {
        if self
            .inner()
            .num_systems_running
            .load(std::sync::atomic::Ordering::Relaxed)
            == 0
        {
            let _ = self._remove_component(entity, T::info_static());
        } else {
            self.inner()
                .cmd_queue
                .push(Cmd::RemoveComponent((entity, T::info_static())));
        }
    }

    fn _remove_component(&self, entity: Entity, component_info: ComponentInfo) -> Result<(), ()> {
        if let Some(Some((archetype, index))) = self.inner().entities.get_mut(*entity as usize) {
            let mut new_archetype = *archetype;
            new_archetype.unset(component_info);

            if *archetype == new_archetype {
                return Result::Err(());
            }

            if !self.inner().stores.contains_key(&new_archetype) {
                if self
                    .inner()
                    .stores
                    .insert(new_archetype, Store::new())
                    .is_some()
                {
                    panic!("")
                }

                let store = unsafe { self.inner().stores.get_mut(archetype).unwrap_unchecked() };
                let new_store = unsafe {
                    self.inner()
                        .stores
                        .get_mut(&new_archetype)
                        .unwrap_unchecked()
                };
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

            let store = unsafe { self.inner().stores.get_mut(archetype).unwrap_unchecked() };
            let new_store = unsafe {
                self.inner()
                    .stores
                    .get_mut(&new_archetype)
                    .unwrap_unchecked()
            };
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
        unsafe { (&mut *self.inner).ctx.assume_init_mut() }
    }

    fn increment_num_running_systems(&self) -> usize {
        let mut current = self
            .inner()
            .num_systems_running
            .load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let new = current + 1;
            match self.inner().num_systems_running.compare_exchange(
                current,
                new,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => return new,
                Err(v) => current = v,
            }
        }
    }

    fn decrement_num_running_systems(&self) -> usize {
        let mut current = self
            .inner()
            .num_systems_running
            .load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let new = current - 1;
            match self.inner().num_systems_running.compare_exchange(
                current,
                new,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => return new,
                Err(v) => current = v,
            }
        }
    }

    pub fn run<'a, Params>(&'a self, mut f: impl System<'a, Params, C>) {
        self.increment_num_running_systems();
        f.run(self);
        let num_running_systems = self.decrement_num_running_systems();

        if num_running_systems == 0 {
            for cmd in &self.inner().cmd_queue {
                match cmd {
                    Cmd::AddComponent((ent, info, component)) => {
                        let _ = self._add_component(*ent, *info, component.as_ref());
                    }
                    Cmd::RemoveComponent((ent, info)) => {
                        let _ = self._remove_component(*ent, *info);
                    }
                };
            }
            self.inner().cmd_queue.clear();
        }
    }

    /// This could return a deleted entity so do not unwrap on ::component<..>(entity)
    pub fn for_each_with_archetype(&self, archetype: Archetype, mut f: impl FnMut(Entity)) {
        unsafe {
            for (store_archetype, store) in self.inner().stores.iter_mut() {
                if archetype == *store_archetype {
                    let len = store.len();
                    let entities = store.get_component_list::<Entity>().unwrap_unchecked();
                    for i in 0..len {
                        f(*entities.read(i))
                    }
                }
            }
        }
    }

    // TODO could be named better no?
    /// This could return a deleted entity so do not unwrap on ::component<..>(entity)
    pub fn for_each_with_archetype_subset(&self, archetype: Archetype, mut f: impl FnMut(Entity)) {
        unsafe {
            for (store_archetype, store) in self.inner().stores.iter_mut() {
                if archetype.subset_of(*store_archetype) {
                    let len = store.len();
                    let entities = store.get_component_list::<Entity>().unwrap_unchecked();
                    for i in 0..len {
                        f(*entities.read(i))
                    }
                }
            }
        }
    }

    // TODO rename
    pub fn num_entities_upper_bound(&self) -> u32 {
        self.inner().entities.len() as u32
    }
}

impl<C: Ctx> Default for World<C> {
    fn default() -> Self {
        Self::new()
    }
}
