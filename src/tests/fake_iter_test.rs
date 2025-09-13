use std::{fmt::Debug, panic::{catch_unwind, AssertUnwindSafe}};

use crate::{
    josie_vec::JosieVec};


pub fn fake_iter_test<T: IntoIterator, A>(test_type:TestType, elements:T)
    where T: IntoIterator<Item = A>, A:Debug
    {
        let test_type @ 
        (lower_bound, upper_bound, panic_index, name) = match test_type{
            TestType::Bounded {bound} => (bound, Some(bound), None, "Bounded"),

            TestType::Unbounded => (0, None, None, "Unbounded"),

            TestType::PartiallyBounded{lower_bound, upper_bound} =>{
                (lower_bound, Some(upper_bound), None, "Partially Bounded")},

            TestType::PanicBounded{bound, panic_index}=>{
                (bound, Some(bound), Some(panic_index), "Panic Bounded")},

            TestType::PanicUnbounded{panic_index}=>{
                (0, None, Some(panic_index), "Panic Unbounded")},

            TestType::PanicPartiallyBounded{lower_bound, upper_bound, panic_index} =>{
                (lower_bound, Some(upper_bound), Some(panic_index), "Panic Partially Bounded")},
        };
        println!("\nRunning Test Type {}, lower bound is {:?}, upper bound is {:?}, panic index is {:?}\n", name, lower_bound, upper_bound, panic_index);

        //initializes new test josievec
        let mut test = JosieVec::new();
        let iter_test = IterTest::new(elements, test_type);
        //checks if there is a panic index in the test initialization
        if let Some(panic_index) = panic_index{
            //if there is wrap the value in panic catch unwind and extend with the iter test iterator
            let _ = catch_unwind(AssertUnwindSafe(||{   
                let _= &mut test.extend(iter_test);
            }));
            assert_eq!(test.len(), panic_index);
            println!("Caught during unwind successfully!\n");
            println!("Josievec Length is {}, expected value after panic is {}, capacity is {}\nRaii Drop guard works for bulk writes\n", test.len(), panic_index, test.capacity())
        }
        //else just run the test straight on the josievec to avoid any possible weirdness with being inside panic catch
        else{
            let _= &mut test.extend(iter_test);
            println!("JosieVec len is {}, capacity is {}\n",test.len(),test.capacity(),);
        }
        println!("contents are\n{:?}", test.as_slice());
}
#[derive(Debug, Clone, Copy)]
pub enum TestType{
    ///iterator with a clear upper bound
    Bounded{
        bound:usize,
    },
    ///iterator with no upper bound, iterates over every element in slice
    Unbounded,
    ///set custom lower bound and upper bound
    PartiallyBounded{
        lower_bound:usize,
        upper_bound:usize,
    },
    PanicBounded{
        bound:usize,
        panic_index:usize,
    },
    //unbounded version, just goes to the upper bound
    PanicUnbounded{
        panic_index:usize
    },
    //Tests Panic behavior, initializes a panic and checks to see if the raii guard does its job by correctly removing all elements after len
    PanicPartiallyBounded{
        lower_bound:usize,
        upper_bound:usize,
        panic_index:usize
    },
}

struct IterTest<T:Iterator<Item = A>, A>{
    //slice of T you want to iterate across
    buf:T,
    //the test type you want to do
    size_hint:(usize, Option<usize>),
    //the optional index at which you will panic
    panic_index:Option<usize>,
    curr:usize,
}

//gross code but its a testing suite lol who cares
impl<T:Iterator<Item = A>, A> IterTest<T,A>{
    fn new<B>(items:B, sizes:(usize, Option<usize>, Option<usize>, &str)) -> Self
        where B:IntoIterator<Item = A, IntoIter = T>, A:Debug
        {
        Self{
            buf:items.into_iter(),
            size_hint:(sizes.0, sizes.1),
            panic_index:sizes.2,
            curr:0,
        }
    }
}

impl<T:Iterator<Item = A>, A> Iterator for IterTest<T,A>{
    type Item = A;

    fn next(&mut self) -> Option<A>{
        if Some(self.curr) == self.panic_index{
            panic!("Boom! Panic!");
        }
        self.curr += 1;
        self.buf.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.size_hint
    }
}