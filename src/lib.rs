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
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

pub trait Resource {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

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

trait QueryParam<'a, T, A> {
    fn info() -> ComponentInfo;
    fn access(world: &'a World, list: Option<&'a ComponentList>, index: usize) -> A;
    fn match_archetype(archetype: &Archetype) -> bool;
}

impl<'a, T: Component + 'static> QueryParam<'a, T, &'a T> for &'a T {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(_: &World, store: Option<&'a ComponentList>, index: usize) -> &'a T {
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
    fn access(_: &World, store: Option<&'a ComponentList>, index: usize) -> &'a mut T {
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
    fn access(_: &World, store: Option<&'a ComponentList>, index: usize) -> Option<&'a T> {
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
    fn access(_: &World, store: Option<&'a ComponentList>, index: usize) -> Option<&'a mut T> {
        unsafe { store.map(|list| list.read_mut::<T>(index)) }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub struct Res<'a, T: Resource + 'static>(&'a T);

impl<'a, T: Resource + 'static> Deref for Res<'a, T> {
    type Target = &'a T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Resource + 'static> QueryParam<'a, T, Res<'a, T>> for Res<'_, T> {
    fn info() -> ComponentInfo {
        ComponentInfo::new(ComponentId(0), 0) // ignored
    }

    #[inline(always)]
    fn access(world: &'a World, _: Option<&'a ComponentList>, _: usize) -> Res<'a, T> {
        match world.get_resource::<T>() {
            Some(r) => Res(r),
            None => panic!("Resource does not exist"),
        }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub struct ResMut<'a, T: Resource + 'static>(&'a mut T);

impl<'a, T: Resource + 'static> Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Resource + 'static> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<'a, T: Resource + 'static> QueryParam<'a, T, ResMut<'a, T>> for ResMut<'_, T> {
    fn info() -> ComponentInfo {
        ComponentInfo::new(ComponentId(0), 0) // ignored
    }

    #[inline(always)]
    fn access(world: &'a World, _: Option<&'a ComponentList>, _: usize) -> ResMut<'a, T> {
        match world.get_resource_mut::<T>() {
            Some(r) => ResMut(r),
            None => panic!("Resource does not exist"),
        }
    }

    fn match_archetype(_: &Archetype) -> bool {
        true
    }
}

pub struct With<T: Component> {
    marker: PhantomData<T>,
}

impl<'a, T: Component + 'static> QueryParam<'a, T, With<T>> for With<T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(_: &'a World, _: Option<&'a ComponentList>, _: usize) -> With<T> {
        With {
            marker: PhantomData::default(),
        }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        archetype.contains(T::info_static())
    }
}

pub struct Without<T: Component> {
    marker: PhantomData<T>,
}

impl<'a, T: Component + 'static> QueryParam<'a, T, Without<T>> for Without<T> {
    fn info() -> ComponentInfo {
        T::info_static()
    }

    #[inline(always)]
    fn access(_: &'a World, _: Option<&'a ComponentList>, _: usize) -> Without<T> {
        Without {
            marker: PhantomData::default(),
        }
    }

    fn match_archetype(archetype: &Archetype) -> bool {
        !archetype.contains(T::info_static())
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
                    for (archetype, store) in world.stores_mut().iter_mut() {
                        if $($param::match_archetype(archetype)) &&+ && true {
                            let mut item_idx = 0;
                            let len = store.len();
                            let entities = store.get_component_list::<Entity>().unwrap();
                            $(let $list = store.get_component_list::<$type>();)+
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

struct WorldInner {
    entities: Vec<Option<(Archetype, usize)>>,
    stores: HashMap<Archetype, Store>,
    free_entities: BTreeSet<Entity>,
    resources: HashMap<TypeId, Box<dyn Resource>>,
}

pub struct World {
    inner: UnsafeCell<WorldInner>,
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        World {
            inner: UnsafeCell::new(WorldInner {
                // Entity(0) is used to mark deleted columns
                entities: vec![None],
                stores: HashMap::new(),
                free_entities: BTreeSet::new(),
                resources: HashMap::new(),
            }),
        }
    }

    fn entities(&self) -> &Vec<Option<(Archetype, usize)>> {
        unsafe { &self.inner.get().as_ref().unwrap().entities }
    }

    fn entities_mut(&self) -> &mut Vec<Option<(Archetype, usize)>> {
        unsafe { &mut self.inner.get().as_mut().unwrap().entities }
    }

    fn stores(&self) -> &HashMap<Archetype, Store> {
        unsafe { &self.inner.get().as_ref().unwrap().stores }
    }

    fn stores_mut(&self) -> &mut HashMap<Archetype, Store> {
        unsafe { &mut self.inner.get().as_mut().unwrap().stores }
    }

    fn free_entities(&self) -> &BTreeSet<Entity> {
        unsafe { &self.inner.get().as_ref().unwrap().free_entities }
    }

    fn free_entities_mut(&self) -> &mut BTreeSet<Entity> {
        unsafe { &mut self.inner.get().as_mut().unwrap().free_entities }
    }

    fn resources(&self) -> &HashMap<TypeId, Box<dyn Resource>> {
        unsafe { &self.inner.get().as_ref().unwrap().resources }
    }

    fn resources_mut(&self) -> &mut HashMap<TypeId, Box<dyn Resource>> {
        unsafe { &mut self.inner.get().as_mut().unwrap().resources }
    }

    pub fn spawn(&self, bundle: &[&dyn Component]) -> Entity {
        unsafe {
            let entity = match self.free_entities_mut().pop_first() {
                Some(e) => e,
                None => Entity(self.entities().len()),
            };

            let mut archetype = Archetype::new();

            for component in bundle {
                archetype.set(component.info());
            }

            archetype.set(Entity::info_static());

            if !self.stores().contains_key(&archetype) {
                self.stores_mut().insert(archetype.clone(), Store::new());
            }

            let store = self.stores_mut().get_mut(&archetype).unwrap();
            let index = store.reserve_index();
            store.write::<Entity>(index, entity);
            for component in bundle {
                store.write_any(component.info(), index, *component);
            }
            match self.entities_mut().get_mut(*entity) {
                Some(p) => *p = Some((archetype, index)),
                None => self.entities_mut().push(Some((archetype, index))),
            }

            entity
        }
    }

    pub fn despawn(&self, entity: Entity) {
        unsafe {
            if entity == Entity(0) {
                return;
            }

            if let Some(Some((archetype, index))) = self.entities().get(*entity) {
                let store = self.stores_mut().get_mut(archetype).unwrap();

                *store.read_mut::<Entity>(*index) = Entity(0);
                store.free_index(*index);

                *self.entities_mut().get_mut(*entity).unwrap() = None;

                self.free_entities_mut().insert(entity);
            }
        }
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        unsafe {
            if let Some(Some((archetype, index))) = self.entities().get(*entity) {
                self.stores().get(archetype).unwrap().try_read::<T>(*index)
            } else {
                None
            }
        }
    }

    pub fn get_component_mut<T: Component + 'static>(&self, entity: Entity) -> Option<&mut T> {
        unsafe {
            if let Some(Some((archetype, index))) = self.entities().get(*entity) {
                self.stores_mut()
                    .get_mut(archetype)
                    .unwrap()
                    .try_read_mut::<T>(*index)
            } else {
                None
            }
        }
    }

    pub fn add_resource<T: Resource + 'static>(&self, resource: T) {
        self.resources_mut()
            .insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get_resource<T: Resource + 'static>(&self) -> Option<&T> {
        self.resources()
            .get(&TypeId::of::<T>())
            .map(|r| r.as_any().downcast_ref().unwrap())
    }

    pub fn get_resource_mut<T: Resource + 'static>(&self) -> Option<&mut T> {
        self.resources_mut()
            .get_mut(&TypeId::of::<T>())
            .map(|r| r.as_mut_any().downcast_mut().unwrap())
    }

    pub fn run<'a, Params>(&'a self, mut f: impl System<'a, Params>) {
        f.run(self)
    }
}
