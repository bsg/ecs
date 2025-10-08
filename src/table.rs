use core::panic;
use std::{
    collections::BTreeSet,
    mem,
    ptr::{drop_in_place, null_mut},
};

use crate::component::{Component, ComponentId, Metadata};

pub(crate) struct Column {
    data: *mut u8,
    stride: usize,
    cap: usize,
}

impl Column {
    pub fn new(item_size: usize) -> Self {
        Column {
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
            .copy_from_nonoverlapping(mem::transmute_copy(&val), val.metadata().size());
    }

    #[inline(always)]
    pub unsafe fn destroy<T: Component + 'static>(&self, idx: usize) {
        drop_in_place::<T>(self.data.add(self.stride * idx).cast());
    }

    #[inline(always)]
    pub unsafe fn copy_item_from_column(
        src: &Column,
        dst: &mut Column,
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

pub(crate) struct Table {
    cols: Vec<Option<Column>>,
    end_index: usize,
    free_indices: BTreeSet<usize>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            cols: (0..128).map(|_| None).collect(), // TODO resize on demand
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

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn read_mut<T: Component + 'static>(&mut self, entity_index: usize) -> &mut T {
        self.cols
            .get_mut(T::metadata_static().id().0 as usize)
            .unwrap()
            .as_mut()
            .unwrap()
            .read_mut::<T>(entity_index)
    }

    pub unsafe fn try_read<T: Component + 'static>(&self, entity_index: usize) -> Option<&T> {
        self.cols
            .get(T::metadata_static().id().0 as usize)
            .unwrap()
            .as_ref()
            .map(|col| col.read::<T>(entity_index))
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn try_read_mut<T: Component + 'static>(
        &mut self,
        entity_index: usize,
    ) -> Option<&mut T> {
        self.cols
            .get_mut(T::metadata_static().id().0 as usize)
            .unwrap()
            .as_mut()
            .map(|col| col.read_mut::<T>(entity_index))
    }

    pub unsafe fn write<T: Component + 'static>(&mut self, entity_index: usize, val: T) {
        if self
            .cols
            .get(T::metadata_static().id().0 as usize)
            .unwrap()
            .is_none()
        {
            self.cols
                .get_mut(T::metadata_static().id().0 as usize)
                .unwrap()
                .replace(Column::new(T::metadata_static().size()));
        }
        self.cols
            .get_mut(T::metadata_static().id().0 as usize)
            .as_mut()
            .unwrap()
            .as_mut()
            .unwrap()
            .write(entity_index, val);
    }

    pub unsafe fn write_any(
        &mut self,
        metadata: Metadata,
        entity_index: usize,
        val: &dyn Component,
    ) {
        if self.cols.get(metadata.id().0 as usize).unwrap().is_none() {
            self.cols
                .get_mut(metadata.id().0 as usize)
                .unwrap()
                .replace(Column::new(metadata.size()));
        }
        self.cols
            .get_mut(metadata.id().0 as usize)
            .as_mut()
            .unwrap()
            .as_mut()
            .unwrap()
            .write_any(entity_index, val);
    }

    pub unsafe fn destroy<T: Component + 'static>(&self, entity_index: usize) {
        if let Some(col) = self
            .cols
            .get(T::metadata_static().id().0 as usize)
            .unwrap()
            .as_ref()
        {
            col.destroy::<T>(entity_index)
        }
    }

    pub unsafe fn add_column_by_id(&mut self, id: ComponentId, size: usize) {
        self.cols
            .get_mut(id.0 as usize)
            .unwrap()
            .replace(Column::new(size));
    }

    pub unsafe fn get_column_by_id(&self, id: ComponentId) -> Option<&Column> {
        self.cols.get(id.0 as usize).unwrap().as_ref()
    }

    pub unsafe fn get_column<T: Component + 'static>(&self) -> Option<&Column> {
        self.get_column_by_id(T::metadata_static().id())
    }

    pub unsafe fn get_column_by_id_mut(&mut self, id: ComponentId) -> Option<&mut Column> {
        self.cols.get_mut(id.0 as usize).unwrap().as_mut()
    }

    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.cols
            .get(T::metadata_static().id().0 as usize)
            .unwrap()
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, Table};
    use crate::{self as ecs, component, Component};

    #[component]
    struct A(u32);

    #[component]
    struct B(&'static str);

    #[test]
    fn column_write_read() {
        let mut col = Column::new(A::metadata_static().size());
        unsafe {
            col.write(0, A(0));
            col.write(1, A(1));
            col.write(100000, A(2));

            assert_eq!(col.read::<A>(0).0, 0);
            assert_eq!(col.read::<A>(1).0, 1);
            assert_eq!(col.read::<A>(100000).0, 2);
        }
    }

    #[test]
    fn table_write_read() {
        let mut table = Table::new();
        unsafe {
            table.write(0, A(100u32));
            table.write(0, B("100"));
            table.write(1, A(101u32));
            table.write(1, B("101"));
            table.write(100000, A(102u32));
            table.write(100000, B("102"));

            assert_eq!(table.read_mut::<A>(0).0, 100u32);
            assert_eq!(table.read_mut::<B>(0).0, "100");
            assert_eq!(table.read_mut::<A>(1).0, 101u32);
            assert_eq!(table.read_mut::<B>(1).0, "101");
            assert_eq!(table.read_mut::<A>(100000).0, 102u32);
            assert_eq!(table.read_mut::<B>(100000).0, "102");
        }
    }
}
