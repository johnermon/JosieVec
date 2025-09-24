// ===============================
//       JOSIEVEC
// -------------------------------
// Learning project to implement my own growable vector
//  not a replacement for any other data type, just for learning
// -------------------------------

use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::{copy, drop_in_place, slice_from_raw_parts_mut, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};

//Declares the mods for josievec
pub mod josievec_drain;
pub mod josievec_extend;
pub mod josievec_iter;

///JosieVec is the Vector
#[derive(Debug)]
pub struct JosieVec<T> {
    pub(crate) buf: RawJosieVec<T>,
    pub(crate) len: usize,
}

///RawJosieVec contains the nonnull pointer and the capacity
#[derive(Debug)]
pub(crate) struct RawJosieVec<T> {
    pub(crate) ptr: NonNull<T>,
    pub(crate) cap: usize,
}
//Public methods
impl<T> JosieVec<T> {
    ///Constructor for JosieVec, Creates an uninitialized JosieVec with capacity of zero
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    ///Constructor for JosieVec with preallocated capacity. Creates a new JosieVec and Reallocs
    ///internal allocaton to be the capacity specified
    pub fn with_capacity(cap: usize) -> Self {
        //creates new default josievec
        let mut temp = Self::default();
        //runs realloc internal to realloc with capacirt specified in cap
        unsafe { temp.realloc_internal(cap) }
        //returns temp
        temp
    }

    ///Pushes an element to JosieVec, incrementing the length by one. If JosieVec is out of
    ///capacity, triggers ammortized growth, doubleing the allocation of the vector
    pub fn push(&mut self, element: T) {
        //if capacity is equal to length then double capacity
        if self.buf.cap == self.len {
            //sets reserves memory equal to current 2 times the length
            self.grow_amortized();
        }
        unsafe { self.push_internal(element) }
    }

    ///Pops last value from JosieVec. If there are no more elements left in the vector, returns
    ///None
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            //returns None early if you try to pop at length zero
            return None;
        }
        //decrements length counter by 1
        self.len -= 1;
        //returns the element at the current length
        Some(unsafe { self.buf.ptr.add(self.len).read() })
    }

    ///Removes element at index, returns to the caller, and appends tail to make JosieVec
    ///contiguous again.
    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("Tried to remove out of bounds element")
        }
        //decrements length counter by one
        self.len -= 1;
        //sets variable out to be equal to the element at the supplied index
        unsafe {
            let out = self.buf.ptr.add(index).read();
            //copies the data from index +1 to end of josievec to be at index.
            copy(
                self.buf.ptr.as_ptr().add(index + 1),
                self.buf.ptr.as_ptr().add(index),
                self.len - index,
            );
            //returns
            return out;
        }
    }

    ///clears all elelmets on the JosieVec.
    #[inline(always)]
    pub fn clear(&mut self) {
        unsafe {
            //drops in place all elements in the vector
            drop_in_place(from_raw_parts_mut(self.as_mut_ptr(), self.len));
        }
        //sets length to zero
        self.len = 0;
    }

    ///Reserves at least enough capacity for the number of elements specified
    #[inline(always)]
    pub fn reserve(&mut self, cap: usize) {
        //sets new capacity variable to be equal to current capacity
        let mut new_capacity: usize = self.buf.cap;
        //while the current length plus the capacity is greater than the new capacity keep doubling until you get a capacity that is greater
        while self.len + cap > new_capacity {
            match new_capacity {
                //new capacity is set to one if size is currently zero
                0 => new_capacity = 1,
                //new capacity is bitshifted by one if it anything else
                _ => new_capacity <<= 1,
            }
        }
        //reallocs with the value of new capacity
        unsafe { self.realloc_internal(new_capacity) }
    }

    ///Reserves capacity for exactly this number of elements
    #[inline(always)]
    pub fn reserve_exact(&mut self, cap: usize) {
        if cap > self.buf.cap {
            unsafe { self.realloc_internal(cap) }
        }
    }

    ///Shrinks JosieVec
    #[inline(always)]
    pub fn shrink_to(&mut self, cap: usize) {
        self.truncate(cap);
        unsafe { self.realloc_internal(cap) }
    }

    ///Shrinks capacity to fit current length
    #[inline(always)]
    ///Shrinks to fit size of alloc
    pub fn shrink_to_fit(&mut self) {
        self.shrink_to(self.len);
    }

    ///Unsafe Function: Sets len to a specified length. unsafe because JosieVec cant uphold
    ///invariants regading whether or not len corresponds to real length.
    #[inline(always)]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    ///truncates JosieVec to length specified
    #[inline(always)]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            unsafe {
                drop_in_place(from_raw_parts_mut(
                    self.buf.ptr.as_ptr().add(len),
                    self.len - len,
                ));
            }
            self.len = len;
        }
    }

    ///Returns slice of Current JosieVec Contents
    #[inline(always)]
    pub const fn as_slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.as_ptr(), self.len) }
    }

    //Returns mutable slice of current JosieVec Contents
    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    ///Creates boxed slice from joseievec
    #[inline(always)]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        //sets self to manually drop so that drop doesnt run.
        let mut drop = ManuallyDrop::new(self);
        unsafe {
            //creates new slice from the raw parts of josievex
            return Box::from_raw(slice_from_raw_parts_mut(drop.as_mut_ptr(), drop.len));
        }
    }

    ///Outputs length of elements currently held
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    ///Outputs current capacity
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.buf.cap
    }

    ///Exposes JosieVec Raw pointer
    pub const fn as_ptr(&self) -> *const T {
        self.buf.ptr.as_ptr()
    }
    ///Exposes Mutable Pointer to JosieVec buffer
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.buf.ptr.as_mut() }
    }
}

//Internal Methods
impl<T> JosieVec<T> {
    ///Pushes withoug any checks. Bounds Checks are iplemented in public facing methods
    #[inline(always)]
    unsafe fn push_internal(&mut self, element: T) {
        //pushes to the JosieVector
        unsafe {
            //writes element to pointer offset by length
            self.buf.ptr.as_ptr().add(self.len).write(element);
        }
        //increments length counter
        self.len += 1;
    }

    ///Grows the vector by power of 2
    #[inline]
    fn grow_amortized(&mut self) {
        //if capacity is 0 then set capacity to 1
        if self.buf.cap == 0 {
            unsafe { self.realloc_internal(1) }
            return;
        }
        //bitshifts self capacity one to the left to double the value
        unsafe {
            self.realloc_internal(self.buf.cap << 1);
        }
    }

    ///Reallocates the Underlying JosieVec allocation
    unsafe fn realloc_internal(&mut self, cap: usize) {
        unsafe {
            //Reallocs Sets josievec Pointer to be the result of this match statement

            //If current cap == 0 and the capacity set to is not equal to zero then allocate a brand new pointer for the JosieVec
            self.buf.ptr = if cap != 0 && self.buf.cap == 0 {
                NonNull::new(
                    //initializes new memory allocation for the josievec
                    alloc(Layout::array::<T>(cap).expect("Overflow")) as *mut T,
                )
            }
            //if the capacity of self is not zero realloc with the size specified. The only way in which it is
            //possible to have a dangling pointer is if the capacity is equal to zero so this is always a safe operation
            else if cap != 0 {
                //creates new non null pointer
                NonNull::new(
                    //reallocates existing memory allocation
                    realloc(
                        self.buf.ptr.as_ptr() as *mut u8,
                        Layout::array::<T>(self.buf.cap).expect("Overflow"),
                        cap * size_of::<T>(),
                    ) as *mut T,
                )
            }
            //if you are reallocating to zero, dealloc the current pointer and then create a new non null dangling pointer
            else if cap == 0 && self.buf.cap != 0 {
                dealloc(
                    self.buf.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.buf.cap).expect("Overflow"),
                );
                Some(NonNull::dangling())
            }
            //the only case this runs is if the current size is zero and you are requesting a resize to zero.
            //in this case just return the already existing pointer in the struct, no need to construct a new one. if this path is taken,
            //I imagine llvm would optimize it away though granted i have no way of proving that without examining machine code which i will not do
            else {
                Some(self.buf.ptr)
            }
            .unwrap_or_else(|| panic!("Tried to realloc with a null pointer"));
            //if NonNull tries to create itself and is given a null pointer then program panics
        }
        //sets capacity to the new cap
        self.buf.cap = cap;
    }
}

impl<T> Deref for JosieVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for JosieVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> Clone for JosieVec<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        //creates new josievec with capacity equal to self
        let mut temp: JosieVec<T> = JosieVec::with_capacity(self.buf.cap);
        //iterates through the josievec and maps the output to a clone
        for element in self.iter().map(|elem| elem.clone()) {
            //pushes each cloned element to the josievec. unsafe function call because guarenteed to push only exact number of elements inside josievec
            unsafe { temp.push_internal(element) }
        }
        //returns temp
        temp

        //old implementation with a serious memory bug, keeping the code here for copy types if i decide to do specializaiton
        // Self{
        // //creates new rawjosievec
        //     buf:RawJosieVec{
        //         //if the capacity is equal to zero use a nonnull dangling pointer
        //         ptr:if self.buf.cap == 0{
        //             NonNull::dangling()
        //         }
        //         //otherwise create a new memory allocation, copy the
        //         else{ unsafe{
        //             let clone_ptr = alloc(Layout::array::<T>(self.buf.cap).expect("Panic")) as *mut T;
        //             copy_nonoverlapping(self.buf.ptr.as_ptr(), clone_ptr, self.len);
        //             NonNull::new(clone_ptr).unwrap_or_else(||panic!("Pointer For Clone of JosieVec was null"))
        //         }},
        //         //sets capacity to be equal to zero
        //         cap:self.buf.cap
        //         },
        //     len:self.len
        // }
    }
}

impl<T> Default for JosieVec<T> {
    fn default() -> Self {
        Self {
            buf: RawJosieVec {
                ptr: NonNull::dangling(),
                cap: 0,
            },
            len: 0,
        }
    }
}

impl<T> Drop for RawJosieVec<T> {
    fn drop(&mut self) {
        unsafe {
            if self.cap != 0 {
                dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).expect("Overflow"),
                );
            }
        }
    }
}

impl<T> Drop for JosieVec<T> {
    fn drop(&mut self) {
        unsafe {
            drop_in_place(from_raw_parts_mut(self.buf.ptr.as_ptr(), self.len));
        }
    }
}

impl<T> FromIterator<T> for JosieVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        //creates new temp josievec
        let mut temp = JosieVec::default();
        //extends the josievec from the iter
        temp.extend(iter);
        //returns temp back to
        temp
    }
}
