use std::{ops::Range, ptr::{copy, drop_in_place}, slice::from_raw_parts_mut};

use crate::josie_vec::JosieVec;
///Iterator for josievec drain, 
pub struct JosieVecDrain<'a, T>{
    //mutable reference to the josievec that is being drained
    pub(crate) josievec:&'a mut JosieVec<T>,
    //the pointer to the first element of the josievec being drained
    pub(crate) start_ptr:*mut T,
    //the actual pointer that is read from
    pub(crate) ptr:*mut T,
    //the end pointer, points to the last pointer in the 
    pub(crate) end_ptr:*mut T,
}

impl<'a, T> Drop for JosieVecDrain<'a, T>{
    fn drop(&mut self){
        unsafe{
            //drops in place any elements owned by the iterator but not iterated through, allows for safe unwind deallocating any
            //items that would otherwise not be deallocated. if it finishes completely without panicing this does nothing
            drop_in_place(from_raw_parts_mut(self.ptr, self.end_ptr.offset_from_unsigned(self.ptr)));
            //elems drained is the number of elements drained from the vector
            let elems_drained = self.ptr.offset_from_unsigned(self.start_ptr);
            //the number of elements present after the porton of the josievec that was drained
            let remaining = self.josievec.len  - self.end_ptr.offset_from_unsigned(self.start_ptr) -1;
            //subtracts erlements drained from length
            self.josievec.len -= elems_drained;
            //copies the elements to the start_ptr
            copy(self.ptr, self.start_ptr, remaining);
        }
    }
}

impl<'a, T> Iterator for JosieVecDrain<'a, T>{
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

impl<T> JosieVec<T>{
    pub fn drain(&mut self, range:Range<usize>)-> JosieVecDrain<T>{
        unsafe{
            if range.start >= self.len || range.end >= self.len{
                panic!("Tried to drain out of range of josievec");
            }
            let ptr = self.buf.ptr.as_ptr().add(range.start);
            JosieVecDrain{
                josievec:self,
                start_ptr:ptr,
                ptr:ptr,
                end_ptr:ptr.add(range.end),
            }
        }
    }
}