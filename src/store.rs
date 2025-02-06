use core::panic;
use std::{
    collections::{BTreeSet, HashMap},
    mem,
    ptr::null_mut,
};

use crate::component::{Component, ComponentId, ComponentInfo};

pub(crate) struct ComponentList {
    data: *mut u8,
    stride: usize,
    cap: usize,
}

impl ComponentList {
    pub fn new(item_size: usize) -> Self {
        ComponentList {
            data: null_mut(),
            stride: item_size,
            cap: 0,
        }
    }

    unsafe fn grow(&mut self, idx: usize) {
        const INITIAL_CAP: usize = 1;

        if idx >= self.cap {
            let new_cap = if idx == 0 { INITIAL_CAP } else { idx * 2 };
            if self.data.is_null() {
                let layout = std::alloc::Layout::array::<u8>(self.stride * new_cap).unwrap();
                self.data = std::alloc::alloc(layout);
            } else {
                let layout = std::alloc::Layout::array::<u8>(self.stride * self.cap).unwrap();
                self.data = std::alloc::realloc(self.data, layout, self.stride * new_cap);
            }
            if self.data.is_null() {
                todo!()
            }
            self.cap = new_cap;
        }
    }

    #[inline(always)]
    pub unsafe fn read<T: Component + 'static>(&self, idx: usize) -> &T {
        &*self.data.add(self.stride * idx).cast::<T>()
    }

    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn read_mut<T: Component + 'static>(&self, idx: usize) -> &mut T {
        &mut *self.data.add(self.stride * idx).cast::<T>()
    }

    #[inline(always)]
    pub unsafe fn write<T: Component + 'static>(&mut self, idx: usize, val: T) {
        self.grow(idx);
        self.data.add(self.stride * idx).cast::<T>().write(val);
    }

    #[inline(always)]
    pub unsafe fn write_any(&mut self, idx: usize, val: &dyn Component) {
        self.grow(idx);
        self.data
            .add(self.stride * idx)
            .copy_from_nonoverlapping(mem::transmute_copy(&val), val.info().size());
    }

    #[inline(always)]
    pub unsafe fn copy_item_from_list(
        src: &ComponentList,
        dst: &mut ComponentList,
        src_idx: usize,
        dst_idx: usize,
    ) {
        if src.stride != dst.stride {
            panic!()
        }

        dst.grow(dst_idx);
        let ptr_src = src.data.add(src.stride * src_idx);
        let ptr_dst = dst.data.add(dst.stride * dst_idx);

        ptr_dst.copy_from_nonoverlapping(ptr_src, dst.stride);
    }

    #[inline(always)]
    pub unsafe fn get_component_size(&self) -> usize {
        self.stride
    }
}

pub(crate) struct Store {
    data: HashMap<ComponentId, ComponentList>,
    end_index: usize,
    free_indices: BTreeSet<usize>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
            end_index: 0,
            free_indices: BTreeSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.end_index
    }

    pub fn reserve_index(&mut self) -> usize {
        if let Some(index) = self.free_indices.pop_first() {
            index
        } else {
            let index = self.end_index;
            self.end_index += 1;
            index
        }
    }

    pub fn free_index(&mut self, index: usize) {
        self.free_indices.insert(index);
    }

    pub unsafe fn read<T: Component + 'static>(&self, entity_index: usize) -> &T {
        self.data
            .get(&T::info_static().id())
            .unwrap()
            .read::<T>(entity_index)
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn read_mut<T: Component + 'static>(&self, entity_index: usize) -> &mut T {
        self.data
            .get(&T::info_static().id())
            .unwrap()
            .read_mut::<T>(entity_index)
    }

    pub unsafe fn try_read<T: Component + 'static>(&self, entity_index: usize) -> Option<&T> {
        self.data
            .get(&T::info_static().id())
            .map(|list| list.read::<T>(entity_index))
    }

    pub unsafe fn try_read_mut<T: Component + 'static>(
        &self,
        entity_index: usize,
    ) -> Option<&mut T> {
        self.data
            .get(&T::info_static().id())
            .map(|list| list.read_mut::<T>(entity_index))
    }

    pub unsafe fn write<T: Component + 'static>(&mut self, entity_index: usize, val: T) {
        if self.data.get(&T::info_static().id()).is_none() {
            self.data.insert(
                T::info_static().id(),
                ComponentList::new(T::info_static().size()),
            );
        }
        self.data
            .get_mut(&T::info_static().id())
            .unwrap()
            .write(entity_index, val);
    }

    pub unsafe fn write_any(
        &mut self,
        component_info: ComponentInfo,
        entity_index: usize,
        val: &dyn Component,
    ) {
        if self.data.get(&component_info.id()).is_none() {
            self.data.insert(
                component_info.id(),
                ComponentList::new(component_info.size()),
            );
        }
        self.data
            .get_mut(&component_info.id())
            .unwrap()
            .write_any(entity_index, val);
    }

    pub unsafe fn add_component_list_by_id(&mut self, id: ComponentId, size: usize) {
        self.data.insert(id, ComponentList::new(size));
    }

    pub unsafe fn get_component_list_by_id(&self, id: ComponentId) -> Option<&ComponentList> {
        self.data.get(&id)
    }

    pub unsafe fn get_component_list<T: Component + 'static>(&self) -> Option<&ComponentList> {
        self.get_component_list_by_id(T::info_static().id())
    }

    pub unsafe fn get_component_list_by_id_mut(
        &mut self,
        id: ComponentId,
    ) -> Option<&mut ComponentList> {
        self.data.get_mut(&id)
    }

    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.data.contains_key(&T::info_static().id())
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentList, Store};
    use crate::{self as ecs, component::Component};
    use codegen::Component;

    #[derive(Component)]
    struct A(u32);

    #[derive(Component)]
    struct B(&'static str);

    #[test]
    fn list_write_read() {
        let mut list = ComponentList::new(A::info_static().size());
        unsafe {
            list.write(0, A(0));
            list.write(1, A(1));
            list.write(100000, A(2));

            assert_eq!(list.read::<A>(0).0, 0);
            assert_eq!(list.read::<A>(1).0, 1);
            assert_eq!(list.read::<A>(100000).0, 2);
        }
    }

    #[test]
    fn store_write_read() {
        let mut store = Store::new();
        unsafe {
            store.write(0, A(100u32));
            store.write(0, B("100"));
            store.write(1, A(101u32));
            store.write(1, B("101"));
            store.write(100000, A(102u32));
            store.write(100000, B("102"));

            assert_eq!(store.read::<A>(0).0, 100u32);
            assert_eq!(store.read::<B>(0).0, "100");
            assert_eq!(store.read::<A>(1).0, 101u32);
            assert_eq!(store.read::<B>(1).0, "101");
            assert_eq!(store.read::<A>(100000).0, 102u32);
            assert_eq!(store.read::<B>(100000).0, "102");
        }
    }
}
