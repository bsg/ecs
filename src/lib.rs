pub use ecs_codegen::component;

pub mod archetype;
pub mod component;
mod table;
mod test;

use archetype::Archetype;
use component::{ComponentId, Metadata};
use table::{Column, Table};

use core::panic;
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
        archetype.set(Entity::metadata_static());
        ArchetypeBuilder(archetype)
    }

    pub fn set<T: Component>(mut self) -> ArchetypeBuilder {
        self.0.set(T::metadata_static());
        self
    }

    pub fn set_from_metadata(&mut self, metadata: Metadata) -> &mut ArchetypeBuilder {
        self.0.set(metadata);
        self
    }

    pub fn build(self) -> Archetype {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Entity(pub u32);

impl Deref for Entity {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

impl Component for Entity {
    fn metadata(&self) -> Metadata {
        Metadata::new(
            ComponentId(0),
            mem::size_of::<Entity>(),
            mem::align_of::<Entity>(),
            "Entity",
        )
    }

    fn metadata_static() -> Metadata
    where
        Self: Sized,
    {
        Metadata::new(
            ComponentId(0),
            mem::size_of::<Entity>(),
            mem::align_of::<Entity>(),
            "Entity",
        )
    }
}

trait QueryParam<'a, T, A> {
    fn access(world: &'a World, col: Option<&'a Column>, index: usize) -> A;
    fn match_archetype(archetype: &Archetype) -> bool;
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a T> for &'a T {
    #[inline(always)]
    fn access(_: &World, col: Option<&'a Column>, index: usize) -> &'a T {
        unsafe { col.unwrap_unchecked().read::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::metadata_static())
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a mut T> for &'a mut T {
    #[inline(always)]
    fn access(_: &World, col: Option<&'a Column>, index: usize) -> &'a mut T {
        unsafe { col.unwrap_unchecked().read_mut::<T>(index) }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::metadata_static())
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Option<&'a T>> for Option<&'a T> {
    #[inline(always)]
    fn access(_: &World, col: Option<&'a Column>, index: usize) -> Option<&'a T> {
        unsafe { col.map(|col| col.read::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Option<&'a mut T>> for Option<&'a mut T> {
    #[inline(always)]
    fn access(_: &World, col: Option<&'a Column>, index: usize) -> Option<&'a mut T> {
        unsafe { col.map(|col| col.read_mut::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub struct With<T: Component> {
    marker: PhantomData<T>,
}

impl<'a, T: Component + 'static> QueryParam<'a, T, With<T>> for With<T> {
    #[inline(always)]
    fn access(_: &'a World, _: Option<&'a Column>, _: usize) -> With<T> {
        With {
            marker: PhantomData,
        }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::metadata_static())
    }
}

pub struct Without<T: Component> {
    marker: PhantomData<T>,
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Without<T>> for Without<T> {
    #[inline(always)]
    fn access(_: &'a World, _: Option<&'a Column>, _: usize) -> Without<T> {
        Without {
            marker: PhantomData,
        }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        !archetype.contains(T::metadata_static())
    }
}

pub trait System<'a, Params> {
    fn run(&mut self, world: &'a World);
}

macro_rules! impl_system {
    ($(($param:ident, $t:ident, $col:ident)),+) => {
        impl<'a, $($param,)+ $($t,)+ F> System<'a, ($($param,)+ $($t,)+)> for F
        where
            $($t: Component + 'static,)+
            $($param: QueryParam<'a, $t, $param>,)+
            F: FnMut($($param,)+),
        {
            fn run(&mut self, world: &'a World) {
                unsafe {
                    for (archetype, table) in world.inner().tables.iter_mut() {
                        if $($param::match_archetype(archetype)) &&+ && true {
                            let len = table.len();
                            let entities = table.get_column::<Entity>().unwrap_unchecked();
                            $(let $col= table.get_column::<$t>();)+
                            for item_idx in 0..len {
                                if entities.read::<Entity>(item_idx).0 != 0 {
                                    self(
                                        $($param::access(world, $col, item_idx),)+
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

pub trait Bundle {
    fn set_archetype(&self, archetype: &mut Archetype);
    #[allow(private_interfaces)]
    fn write_self_to_table(self, index: usize, table: &mut Table);
}

macro_rules! impl_bundle {
    ($(($t:ident, $idx:tt)),+) => {
        impl<$($t,)+> Bundle for ($($t,)+)
        where
            $($t: Component + 'static,)+
        {
            fn set_archetype(&self, archetype: &mut Archetype) {
                $(archetype.set(self.$idx.metadata());)+
            }

            #[allow(private_interfaces)]
            fn write_self_to_table(self, index: usize, table: &mut Table) {
                unsafe { $(table.write_any(self.$idx.metadata(), index, &self.$idx);)+ };
                mem::forget(self);
            }
        }
    }
}

impl<T1: Component + 'static> Bundle for T1 {
    fn set_archetype(&self, archetype: &mut Archetype) {
        archetype.set(self.metadata());
    }

    #[allow(private_interfaces)]
    fn write_self_to_table(self, index: usize, table: &mut Table) {
        unsafe { table.write_any(self.metadata(), index, &self) };
        mem::forget(self);
    }
}

// XXX
impl_bundle!((T1, 0), (T2, 1));
impl_bundle!((T1, 0), (T2, 1), (T3, 2));
impl_bundle!((T1, 0), (T2, 1), (T3, 2), (T4, 3));
impl_bundle!((T1, 0), (T2, 1), (T3, 2), (T4, 3), (T5, 4));
impl_bundle!((T1, 0), (T2, 1), (T3, 2), (T4, 3), (T5, 4), (T6, 5));
impl_bundle!(
    (T1, 0),
    (T2, 1),
    (T3, 2),
    (T4, 3),
    (T5, 4),
    (T6, 5),
    (T7, 6)
);

impl_bundle!(
    (T1, 0),
    (T2, 1),
    (T3, 2),
    (T4, 3),
    (T5, 4),
    (T6, 5),
    (T7, 6),
    (T8, 7)
);

impl_bundle!(
    (T1, 0),
    (T2, 1),
    (T3, 2),
    (T4, 3),
    (T5, 4),
    (T6, 5),
    (T7, 6),
    (T8, 7),
    (T9, 8)
);

impl_bundle!(
    (T1, 0),
    (T2, 1),
    (T3, 2),
    (T4, 3),
    (T5, 4),
    (T6, 5),
    (T7, 6),
    (T8, 7),
    (T9, 8),
    (T10, 9)
);

impl_bundle!(
    (T1, 0),
    (T2, 1),
    (T3, 2),
    (T4, 3),
    (T5, 4),
    (T6, 5),
    (T7, 6),
    (T8, 7),
    (T9, 8),
    (T10, 9),
    (T11, 10)
);

impl_bundle!(
    (T1, 0),
    (T2, 1),
    (T3, 2),
    (T4, 3),
    (T5, 4),
    (T6, 5),
    (T7, 6),
    (T8, 7),
    (T9, 8),
    (T10, 9),
    (T11, 10),
    (T12, 11)
);

enum Cmd {
    AddComponent((Entity, Metadata, Box<dyn Component>)),
    RemoveComponent((Entity, Metadata)),
}

struct WorldInner {
    entities: Vec<Option<(Archetype, usize)>>,
    tables: HashMap<Archetype, Table>,
    free_entities: BTreeSet<Entity>,
    cmd_queue: Vec<Cmd>,
    num_systems_running: AtomicUsize,
}

pub struct World {
    inner: *mut WorldInner,
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        World {
            inner: Box::into_raw(Box::new(WorldInner {
                // Entity(0) is used to mark deleted columns
                entities: vec![None],
                tables: HashMap::new(),
                free_entities: BTreeSet::new(),
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

    #[allow(clippy::mut_from_ref)]
    fn inner(&self) -> &mut WorldInner {
        unsafe { &mut *self.inner }
    }

    // TODO make this safer using a spawn queue?
    // TODO flesh out this doc
    /// # SAFETY
    ///
    /// Spawning inside a system where the archetype of the spawned entity is a subset of the query archetype
    /// is problematic
    pub unsafe fn spawn<B: Bundle>(&self, bundle: B) -> Entity {
        let entity = self
            .inner()
            .free_entities
            .pop_first()
            .unwrap_or(Entity(self.inner().entities.len() as u32));

        let mut archetype = Archetype::new();

        bundle.set_archetype(&mut archetype);

        archetype.set(Entity::metadata_static());

        self.inner()
            .tables
            .entry(archetype)
            .or_insert_with(Table::new);

        let table = unsafe { self.inner().tables.get_mut(&archetype).unwrap_unchecked() };
        let index = table.reserve_index();
        unsafe { table.write::<Entity>(index, entity) };
        bundle.write_self_to_table(index, table);
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
            archetype.set(item.metadata());
        }

        archetype.set(Entity::metadata_static());

        self.inner()
            .tables
            .entry(archetype)
            .or_insert_with(Table::new);

        let table = unsafe { self.inner().tables.get_mut(&archetype).unwrap_unchecked() };
        let index = table.reserve_index();
        unsafe { table.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { table.write_any(item.metadata(), index, &**item) };
        }
        match self.inner().entities.get_mut(*entity as usize) {
            Some(p) => *p = Some((archetype, index)),
            None => self.inner().entities.push(Some((archetype, index))),
        }

        entity
    }

    // TODO tests
    // TODO this is almost identical to spawn(). dedup
    pub fn insert<B: Bundle>(&self, entity: Entity, bundle: B) -> Entity {
        let mut archetype = Archetype::new();

        bundle.set_archetype(&mut archetype);

        archetype.set(Entity::metadata_static());

        self.inner()
            .tables
            .entry(archetype)
            .or_insert_with(Table::new);

        let table = unsafe { self.inner().tables.get_mut(&archetype).unwrap_unchecked() };
        let index = table.reserve_index();
        unsafe { table.write::<Entity>(index, entity) };
        bundle.write_self_to_table(index, table);

        if *entity as usize >= self.inner().entities.len() {
            self.inner().entities.resize(*entity as usize + 1, None);
            // TODO add the slots in the gap to the free list
        }
        self.inner().entities.as_mut_slice()[*entity as usize] = Some((archetype, index));
        self.inner().free_entities.remove(&entity);

        entity
    }

    // TODO ton of code shared again...
    pub fn insert_from_slice_of_boxes(
        &self,
        entity: Entity,
        bundle: &[Box<dyn Component>],
    ) -> Entity {
        let mut archetype = Archetype::new();

        for item in bundle {
            archetype.set(item.metadata());
        }

        archetype.set(Entity::metadata_static());

        self.inner()
            .tables
            .entry(archetype)
            .or_insert_with(Table::new);

        let table = unsafe { self.inner().tables.get_mut(&archetype).unwrap_unchecked() };
        let index = table.reserve_index();
        unsafe { table.write::<Entity>(index, entity) };
        for item in bundle {
            unsafe { table.write_any(item.metadata(), index, &**item) };
        }
        match self.inner().entities.get_mut(*entity as usize) {
            Some(p) => *p = Some((archetype, index)),
            None => self.inner().entities.push(Some((archetype, index))),
        }

        entity
    }

    pub fn despawn(&self, entity: Entity) {
        if entity == Entity(0) {
            return;
        }

        if let Some(Some((archetype, index))) = self.inner().entities.get(*entity as usize) {
            let table = unsafe { self.inner().tables.get_mut(archetype).unwrap_unchecked() };

            *unsafe { table.read_mut::<Entity>(*index) } = Entity(0);
            table.free_index(*index);

            *unsafe {
                self.inner()
                    .entities
                    .get_mut(*entity as usize)
                    .unwrap_unchecked()
            } = None;

            self.inner().free_entities.insert(entity);
        }
    }

    pub fn has_component<T: Component + 'static>(&self, entity: Entity) -> bool {
        if let Some(Some((archetype, _))) = self.inner().entities.get(*entity as usize) {
            unsafe {
                self.inner()
                    .tables
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
                    .tables
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
                    .tables
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
            let _ = self._add_component(entity, T::metadata_static(), &component);
        } else {
            self.inner().cmd_queue.push(Cmd::AddComponent((
                entity,
                T::metadata_static(),
                Box::new(component),
            )));
        }
    }

    fn _add_component(
        &self,
        entity: Entity,
        metadata: Metadata,
        component: &dyn Component,
    ) -> Result<(), ()> {
        if let Some(Some((archetype, index))) = self.inner().entities.get_mut(*entity as usize) {
            let mut new_archetype = *archetype;
            new_archetype.set(component.metadata());

            if *archetype == new_archetype {
                return Result::Err(());
            }

            if let std::collections::hash_map::Entry::Vacant(e) =
                self.inner().tables.entry(new_archetype)
            {
                e.insert(Table::new());

                let table = unsafe { self.inner().tables.get_mut(archetype).unwrap_unchecked() };
                let new_table = unsafe {
                    self.inner()
                        .tables
                        .get_mut(&new_archetype)
                        .unwrap_unchecked()
                };

                for id in 0..128 {
                    unsafe {
                        if let Some(col) = table.get_column_by_id(ComponentId(id as u32)) {
                            if new_archetype.contains_id(id) {
                                new_table.add_column_by_id(
                                    ComponentId(id as u32),
                                    col.get_component_size(),
                                    metadata.align(),
                                )
                            }
                        }
                    };
                }
            }

            let table = unsafe { self.inner().tables.get_mut(archetype).unwrap_unchecked() };
            let new_table = unsafe {
                self.inner()
                    .tables
                    .get_mut(&new_archetype)
                    .unwrap_unchecked()
            };
            let new_index = new_table.reserve_index();

            for id in 0..128 {
                if archetype.contains_id(id) {
                    unsafe {
                        Column::copy_item_from_column(
                            table
                                .get_column_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            new_table
                                .get_column_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            *index,
                            new_index,
                        );
                    }
                }
            }

            unsafe { table.write::<Entity>(*index, Entity(0)) };
            table.free_index(*index);

            unsafe { new_table.write_any(metadata, new_index, component) };

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
            let _ = self._remove_component(entity, T::metadata_static());
        } else {
            self.inner()
                .cmd_queue
                .push(Cmd::RemoveComponent((entity, T::metadata_static())));
        }
    }

    fn _remove_component(&self, entity: Entity, metadata: Metadata) -> Result<(), ()> {
        if let Some(Some((archetype, index))) = self.inner().entities.get_mut(*entity as usize) {
            let mut new_archetype = *archetype;
            new_archetype.unset(metadata);

            if *archetype == new_archetype {
                return Result::Err(());
            }

            if !self.inner().tables.contains_key(&new_archetype) {
                if self
                    .inner()
                    .tables
                    .insert(new_archetype, Table::new())
                    .is_some()
                {
                    panic!()
                }

                let table = unsafe { self.inner().tables.get_mut(archetype).unwrap_unchecked() };
                let new_table = unsafe {
                    self.inner()
                        .tables
                        .get_mut(&new_archetype)
                        .unwrap_unchecked()
                };

                for id in 0..128usize {
                    unsafe {
                        if let Some(col) = table.get_column_by_id(ComponentId(id as u32)) {
                            if new_archetype.contains_id(id) {
                                new_table.add_column_by_id(
                                    ComponentId(id as u32),
                                    col.get_component_size(),
                                    metadata.align(),
                                )
                            }
                        }
                    };
                }
            }

            let table = unsafe { self.inner().tables.get_mut(archetype).unwrap_unchecked() };
            let new_table = unsafe {
                self.inner()
                    .tables
                    .get_mut(&new_archetype)
                    .unwrap_unchecked()
            };
            let new_index = new_table.reserve_index();

            for id in 0..128usize {
                if new_archetype.contains_id(id) {
                    unsafe {
                        Column::copy_item_from_column(
                            table
                                .get_column_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            new_table
                                .get_column_by_id_mut(ComponentId(id as u32))
                                .unwrap_unchecked(),
                            *index,
                            new_index,
                        );
                    }
                }
            }

            unsafe { table.write::<Entity>(*index, Entity(0)) };
            table.free_index(*index);

            *archetype = new_archetype;
            *index = new_index;

            return Result::Ok(());
        }

        Result::Err(())
    }

    pub fn destroy_component<T: Component + 'static>(&self, entity: Entity) {
        unsafe {
            if let Some(Some((archetype, index))) = self.inner().entities.get(*entity as usize) {
                self.inner()
                    .tables
                    .get_mut(archetype)
                    .unwrap_unchecked()
                    .destroy::<T>(*index)
            }
        }
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

    /// # SAFETY
    ///
    /// - Nesting queries with a subset relationship could result in mutable aliasing
    /// ```
    ///   world.run(|foo: &mut Foo, bar: &Bar| {
    ///      world.run(|foo: &Foo /* UB */| {
    ///          ...
    ///      });
    ///   });
    /// ```
    pub unsafe fn run<'a, Params>(&'a self, mut f: impl System<'a, Params>) {
        self.increment_num_running_systems();
        f.run(self);
        let num_running_systems = self.decrement_num_running_systems();

        if num_running_systems == 0 {
            for cmd in &self.inner().cmd_queue {
                match cmd {
                    Cmd::AddComponent((ent, metadata, component)) => {
                        let _ = self._add_component(*ent, *metadata, component.as_ref());
                    }
                    Cmd::RemoveComponent((ent, metadata)) => {
                        let _ = self._remove_component(*ent, *metadata);
                    }
                };
            }
            self.inner().cmd_queue.clear();
        }
    }

    /// This could return a deleted entity so do not unwrap on ::component<..>(entity)
    pub fn for_each_with_archetype(&self, archetype: Archetype, mut f: impl FnMut(Entity)) {
        unsafe {
            for (table_archetype, table) in self.inner().tables.iter_mut() {
                if archetype == *table_archetype {
                    let len = table.len();
                    let entities = table.get_column::<Entity>().unwrap_unchecked();
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
            for (table_archetype, table) in self.inner().tables.iter_mut() {
                if archetype.subset_of(*table_archetype) {
                    let len = table.len();
                    let entities = table.get_column::<Entity>().unwrap_unchecked();
                    for i in 0..len {
                        f(*entities.read(i))
                    }
                }
            }
        }
    }

    pub fn num_entities_max(&self) -> u32 {
        self.inner().entities.len() as u32
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
