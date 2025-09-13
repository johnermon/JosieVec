use std::{alloc::{dealloc, Layout}, mem::{take, ManuallyDrop}, ptr::{drop_in_place}, slice::from_raw_parts_mut};

use super::JosieVec;

pub struct JosieVecIterRef<'a, T>{
    pub(crate) slice:&'a [T],
}

pub struct JosieVecIterMut<'a, T>{
    pub(crate) slice:&'a mut [T],
}

//macro for into itertayot Traits for 
macro_rules! into_iterator {
    ($item_type:ty,$into_iter_type:ty,$iter_type:ident) => {
        type Item = $item_type;

        type IntoIter =$into_iter_type;

        fn into_iter(self) -> Self::IntoIter{
            self.$iter_type()
        }
    };
}

impl<'a, T> IntoIterator for &'a JosieVec<T> {
    into_iterator!{&'a T,JosieVecIterRef<'a, T>,iter}
}

impl<'a, T> IntoIterator for &'a mut JosieVec<T> {
    into_iterator!{&'a mut  T,JosieVecIterMut<'a, T>,iter_mut}
}

///Macro for firt output element function on mutable and immutable iteratoes
macro_rules! output_first_element {
    ($src:expr,$dst:expr, $split_type:ident) => {
        if let Some((first, rest)) = $src.$split_type(){
            $dst = rest;
            Some(first)
        }else{
            None
        }
    };
}

impl<'a, T> Iterator for JosieVecIterRef<'a, T>{
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        output_first_element!(self.slice, self.slice, split_first)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.slice.len();
            (size,Some(size))
    }
}

impl<'a, T> Iterator for JosieVecIterMut<'a, T>{
    type Item = &'a mut T;
    
    fn next(&mut self) -> Option<Self::Item> {
        let take = take(&mut self.slice);
        output_first_element!(take,self.slice, split_first_mut)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.slice.len();
        (size,Some(size))
    }
}



impl<T> JosieVec<T>{     
    #[inline(always)]
    pub const fn iter(&self) -> JosieVecIterRef<T>{
        JosieVecIterRef {slice:self.as_slice()}
    }
    #[inline(always)]
    pub const fn iter_mut(&mut self) -> JosieVecIterMut<T>{
        JosieVecIterMut {slice:self.as_mut_slice()}
    }
}


pub struct JosieVecIter<T>{
    pub(crate) start_ptr:*mut T,
    pub(crate) ptr:*mut T,
    pub(crate) end_ptr:*mut T,
}

impl<T> Drop for JosieVecIter<T>{
    fn drop(&mut self){
        unsafe{
            //drops in place any elements owned by the iterator but not iterated through, allows for safe unwind deallocating any
            //items that would otherwise not be deallocated. if it finishes completely without panicing this does nothing
            drop_in_place(from_raw_parts_mut(self.ptr, self.end_ptr.offset_from_unsigned(self.ptr)));
            //if the iterator was consuming then deallocate the memory that was initially allocated to josievec by calculating the capacity.
            dealloc(self.start_ptr as *mut u8, Layout::array::<T>(self.end_ptr.offset_from_unsigned(self.start_ptr)).expect("Overflow"));
        }
    }
}


impl<T> IntoIterator for JosieVec<T>{
    type Item = T;

        type IntoIter = JosieVecIter<T>;

        fn into_iter(self) -> JosieVecIter<T>{
            let to_drop = ManuallyDrop::new(self);
            //creates new josieVecIter  with ownership of raw pointer to data, an iteration counter and the length of the current buffer
            let ptr = to_drop.buf.ptr.as_ptr();
            JosieVecIter{
                start_ptr:ptr,
                end_ptr: unsafe{ptr.add(to_drop.len)},
                ptr,
            }
        }
}

impl<T> Iterator for JosieVecIter<T>{
    type Item = T;

    fn next(&mut self) -> Option<T>{
        if self.ptr == self.end_ptr{
            return None;
        }
        unsafe{
            let out = self.ptr.read();
            self.ptr = self.ptr.add(1);
            Some(out)
        }
        
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = unsafe{self.end_ptr.offset_from_unsigned(self.start_ptr)};
        (size, Some(size))
    }
    
}