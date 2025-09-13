
// ===============================
// EXTEND IMPLEMENTATION
// -------------------------------
// Extend for josievec uses size_hint on iter in order to intellegently reallocate buffer
// Roughly 0(1) ish reallocs
// -------------------------------

use std::{
    mem::ManuallyDrop,
};

use super::JosieVec;

//raii drop guard for josievec, on drop drops in place any elements initialized before 
struct JosieVecGuard<'a, T>{
    //the length of the josievec at start of push
    start_len:usize,
    //mutable reference to the josievec
    josievec:&'a mut JosieVec<T>,
    //start pointer and end pointer for the data in the struct
    start_ptr:*mut T,
    ptr:*mut T,
}

impl<'a, T> JosieVecGuard<'a, T>{
    //arms the josievec guard via saving all relevent fields to the struct
    fn arm(josievec:&'a mut JosieVec<T>) -> Self{
        //constructs pointer from the josievecs pointer offset by the current length
        let ptr = unsafe{
            josievec.buf.ptr.as_ptr().add(josievec.len)
        };

        Self{
            start_len:josievec.len,
            start_ptr:ptr,
            ptr:ptr,
            josievec:josievec,

        }
    }
    #[inline(always)]
    //disarms self by converting it to a manually dropped so it doesnt run destructor
    const fn commit(self){
        //will convert self to manually drop so that drop code doesnt run on disarm.
        let _ = ManuallyDrop::new(self);
    }
}

impl<'a, T> Drop for JosieVecGuard<'a, T>{
    fn drop(&mut self) {
        //the only time this code runs is if the push never finished because of a panic.
        self.josievec.len = self.start_len + unsafe{
            self.ptr.offset_from_unsigned(self.start_ptr)
        };
    }
}

macro_rules! push_iter {
    (bounded $josievec:expr, $bound:expr, $iterator:expr) => {
        //sets reserves memory equal to current size + upper bound of possible iterator values
        $josievec.reserve($bound);
        let mut guard = JosieVecGuard::arm($josievec);
        unsafe{
            //pushes contents of iterator to josievec
            for element in $iterator.take({$bound}){
                guard.ptr.write(element);
                guard.ptr = guard.ptr.add(1);
            }
        }
        //disarms josievec guard
        guard.commit();
        //sets length to final value
        $josievec.len += $bound;
    };
    (unbounded $josievec:expr, $iterator:expr)=>{
        'outer: loop{
            //creates variable for the number of elements by subtracting current length from bounds
            let elems = $josievec.buf.cap - $josievec.len;
            //arms josievec guard
            let mut guard = JosieVecGuard::arm($josievec);
            unsafe{
                //counts up to i elements. hot loop, 1 branch per iteration and pointer bump for indexing. fast af extend.
                for i in 0..elems{
                    //if iterator returns some then write it to the pointer
                    if let Some(element) = $iterator.next(){
                        //if i==5{break 'outer}
                        //writes to pointer
                        guard.ptr.write(element);
                        //increments pointer by one
                        guard.ptr = guard.ptr.add(1);
                    }
                    //else set josievec len to equal to the bounds - the elements + the current iteration, setting length properly.
                    else{
                        //commits changes to josievec
                        guard.commit();
                        //sets length properly given data
                        $josievec.len = ($josievec.buf.cap - elems + i);
                        //breaks outer loop
                        break 'outer;
                    }
                }
                //comitts changes to the josievec
                guard.commit();
                //sets the length to be equal to current cap
                $josievec.len = $josievec.buf.cap;
                //grows the vector
                $josievec.grow_amortized();
            }
        };
    }
}

pub enum ExtendType{
    Exact(usize),
    Ammortized,
}

impl<T> JosieVec<T>{
    ///Bulk extend Guaured is an unsafe function that attempts to eliminate as many invariants from bulk unsafe writes as humanly possible
    ///while remaining a fast pointer walk. 
    ///Usage:Certain Invariants must be upheld for this function to be used safely. Number one, you must write valid T to the Josievec.
    ///function will do nothing to stop you from writing garbage data onto the raw pointer.
    ///Second:Only increment the pointer. the length setting logic will underflow if you hand back a decremented pointer, causing a segfault.
    ///this could be fine if you remember to bump your pointer at the end and the elements you are writing are trivial but if the elements own
    ///data on the heap the function has no way of guarenteeing overwritten elements drop.
    ///USAGE: the ptr pointer is the one you write to and bump. any updates to it are written to the pointer in JosievecGuard struct.
    ///When you run function with elems, it creats two other pointers. end ptr is a pointer to the exact place in memory you need to stop at
    ///for your elements input, and max pointer is the max capacity allocated on resize. if you will be writing an exact amount of elements,
    ///use the end ptr. if you are unbounded, use the max_ptr and break and restart the function when you run out of capacity.
    ///on function completion, the Josievecguard runs its drop code which sets the length to the exact number of elements written by getting
    ///offset of the write pointer from the start pointer held inside JosieVecGuard, so as long as you account for previously mentioned invariants
    ///you dont need to worry about setting the length manually. if code inside the loop panics JosieVecGuard will also make sure the capacity is 
    ///consistent with elements written. For Example on how to implement this, check out fibonacci_test.
    pub unsafe fn bulk_extend_guarded<F>(&mut self, extend_type:ExtendType, f:F)
    where F: FnOnce(&mut *mut T,*mut T){
        //creates new variable elems
        let elems:usize;
        //matches extend type to either end Exact or ammortized
        match extend_type{
            ExtendType::Exact(elements)=>{
                //reserves enough capacity for elems that will be written
                self.reserve(elements);
                //sets number of elems to be equal to the number of elements that will be written
                elems = elements;
                
            },
            ExtendType::Ammortized=>{
                //if current len is equal to the buffer cap then grow ammortized
                if self.len == self.buf.cap{
                    self.grow_amortized();
                }
                //sets elems to the cap minus current length
                elems = self.buf.cap - self.len;
            }
        }
        //creates end pointer that is pointing at the max capacity
        let mut guard = JosieVecGuard::arm(self);
        //sets end_ptr to be equal to the pointer location of the max elems you are pushing into the vec
        //the pointer you use to write to
        let ptr = &mut guard.ptr;
        unsafe{
        let end_ptr = ptr.add(elems);
        //passes in a mutable reference to the raii drop guards end ptr to manipulate along with a pointer pointing to the last valid
        //memory location in the buffer
        f(ptr, end_ptr);
        }
        //never comitts the josievec guard, vec guard itself handles any invariants regarding actual elements written. so long as
        //Code writes valid t and doesnt advance pointer backwards, the guard will set length to max number of elements written automatically
    }

}

//implementation for extend for JosieVec
impl<T> Extend<T> for JosieVec<T>{
    fn extend<A>(&mut self, iter: A)
    where A: IntoIterator<Item = T>{
        //creates new iterator from iter
        let mut iterator = iter.into_iter();
        //gets possible ranges for iter based off of size_hint
        let iter_ranges = iterator.size_hint();
        //checks for an upper bound on the iterator
        match iter_ranges{
            //if upper bound for iterator is zero return straight away
            (_, Some(0)) => return,
            //if there is a lower bound and an upper bound and both are equal allocate enough capacity for the upper bound
            (lower_bound, Some(upper_bound)) if lower_bound == upper_bound =>{
                 push_iter!(bounded self, upper_bound, iterator);},
            //any other cases where there is a lower and an upper bound first allocate enough capacity for lower bound,
            //then allocate enough capacity for upper bound after that allocate enough space for upper bounds
            (lower_bound, Some(upper_bound)) =>{
                push_iter!(bounded self, lower_bound, &mut iterator);
                push_iter!(bounded self, upper_bound - lower_bound, iterator);
            },
            //if there is no upper bound then just standard push behavior 
            (0, None) =>{
                push_iter!(unbounded self, iterator);
            }
            //if there is a lower bound but no upper bound, first allocate enough capacity for the lower bound then resume unbounded afterwards
            (lower_bound, None) =>{
                push_iter!(bounded self, lower_bound, &mut iterator);
                push_iter!(unbounded self, iterator);
            },
        }
    }
}
