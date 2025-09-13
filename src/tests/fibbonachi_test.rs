use std::{arch::asm};

use crate::josie_vec::{josievec_extend::ExtendType, *};
impl JosieVec<usize>{
    ///Tests Bulk extend guarded function. fibonacci push just calculates a fibonacci numbers and uses the ptr given by the closure to write to the buffer directly.
    ///the only bookkepeing for how many elements written is the state of the pointer itself, the raii guard calculates length on its drop. additionally,
    ///hot loops like this where you want to avoid calculating pointer offsets alltogether and just push in bulk is the perfect use for this function.
    ///you can do all the calculations and let the write pointer bookeep for you.
    pub fn fibonacci_push(&mut self, elems:usize){
        unsafe{
            self.bulk_extend_guarded(ExtendType::Exact(elems), |ptr, end_ptr|{
                //const ptr size is equal to the size of usize, passed into asm functions in order to have pointer advance logic consistant across 64 and 32 bit.
                const PTRSIZE:usize = size_of::<usize>();
                //Arm assembly version
                #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
                asm!(                       //pseudocode
                                            //fn fibonacci_asm(ptr:*mut T, max:usize){
                    "mov {a}, #0",              //let a = 0;
                    "mov {b}, #1",              //let b = 1;               
                    "0:",                       //loop{
                    "add {b}, {a}, {b}",            //b = a + b;
                    "str {a}, [{ptr}]",             //*ptr = b;
                    "add {ptr}, {ptr}, {ptr_len}",  //ptr = ptr.add(1);
                    "cmp {ptr}, {end}",               //if ptr == end{
                    "b.eq 1f",                          //break;
                                                    //}
                    "add {a}, {a}, {b}",            //a = a + b;
                    "str {b}, [{ptr}]",             //*ptr = b;
                    "add {ptr}, {ptr}, {ptr_len}",  //ptr = ptr.add(1);
                    "cmp {ptr}, {end}",               //if ptr != end{
                    "b.ne 0b",                          //continue;
                    "1:",                           //}
                                                //}
                                            //}
                    //sets end to be equal to the end ptr
                    end = in(reg) end_ptr,
                    //sends in size of usize for pointer size calculations
                    ptr_len = in(reg) PTRSIZE, 
                    //passes pointer to JosieVec into asm
                    ptr = inout(reg) *ptr,
                    // creates new register a and b with 1 and 0 in them to initialize fibonacci calculation
                    a = out(reg)_, 
                    b = out(reg)_,
                    //sets to nostack becuase stack is not modified at all by program
                    options(nostack)
                );
                //x86 assembly version
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                asm!(                   //pseudocode
                                        //fn fibonacci_asm(ptr:*mut T, max:usize){
                    "mov {a}, 0",           //let a = 0;
                    "mov {b}, 1",           //let b = 1;
                    "2:",                   //loop{
                    "add {b}, {a}",             //b = a + b;
                    "mov [{ptr}], {a}",         //*ptr = a;
                    "add {ptr}, {ptr_len}",     //ptr = ptr.add(1);
                    "cmp {ptr}, {end}",           //if ptr == end{
                    "je 3f",                        //break;
                                                //}
                    "add {a}, {b}",             //a = a + b;
                    "mov [{ptr}], {b}",         //*ptr = a;
                    "add {ptr}, {ptr_len}",     //ptr = ptr.add(1);
                    "cmp {ptr}, {end}",           //if ptr == end{
                    "jne 2b",                       //continue;
                    "3:",                       //}
                                            //}
                                        //}
                    //sets max to be equal to the
                    end = in(reg) end_ptr,
                    //sends in size of usize for pointer size calculations
                    ptr_len = in(reg) PTRSIZE, 
                    //passes pointer to JosieVec into asm
                    ptr = inout(reg) *ptr,
                    // creates new register a and b with 1 and 0 in them to initialize fibonacci calculation
                    a = out(reg)_, 
                    b = out(reg)_,
                    //sets to nostack becuase stack is not modified at all by program
                    options(nostack)
                );
            });
        }
    }
}

    