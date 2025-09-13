#![allow(dead_code)]
use std::iter::repeat;
use std::{any::type_name,};
use rand::Rng;
pub mod fake_iter_test;
pub mod fibbonachi_test;

use crate::josievec;
use crate::tests::fake_iter_test::{fake_iter_test, TestType};
use crate::josie_vec::JosieVec;

pub fn josievec_test(){
    fake_iter_test(TestType::Unbounded, 
        ["test";6]);
    fake_iter_test(TestType::PanicUnbounded { 
        panic_index: 5 },
         [10;10]);
    fake_iter_test(TestType::Bounded { bound: 6 }, 
        ["test";6]);
    fake_iter_test(TestType::PanicBounded { bound: 10, panic_index: 7 },
         [10;10]);
    let mut josievec = init_test_vec::<&str>();
    //pushes 9 strs to the josievec then displays it as a slice, checks the capacity on each push to test growth behavior
    push_then_pop(&mut josievec);
    //reserves capacacity then extends the josievec with an array, shrinks to fit and then pops all items off +1 to test if it gracefully handles
    //popping at length 0
    reserve_and_shrink(&mut josievec);
    //performs a clone of josievec and then writes to each independently and prints the results.
    clone_then_check(&mut josievec);
    //saves 10 strings to the josievec, then chunks it and checks the results 
    slice_and_chunk(&mut josievec);
    //does a fake test for bounded iter to check how josievec resizes on bounded iterator
    //fake_iter_test(TestType::FakeBounded {});
    //tests boxed slice behavior
    into_boxed_slice_test();
    //Tests the use of the macro for creation of a josievec
    macro_test();
    //calculates fibbonachi numbers and saves them to josievec and prints them
    //creates new josievec with float type 
    //calculates averages of numbers 1..max where averages then next
    mutate_in_place_averaging(10);
    //uses bulk guarded extend method to push fibonacci numbers to josievec directly using inline asm.
    fibonacci_test();
    //creates a josievec full of clones of josievecs, mutates each josievec and then outputs the result
    mutate_nested_clones();
    //creates new josievec, drains elements from it then collects that into another  josievec. prints the original josievec before and afte
    //drain and the drained slice
    drain_then_collect();
}

fn init_test_vec<T>()->JosieVec<T>{
    println!("\nCreating JosieVec of element type {}\n\n", type_name::<T>());
    let josievec:JosieVec<T> = JosieVec::new();
    josievec
}

fn push_then_pop(josievec:&mut JosieVec<&str>){
    println!("\nPush then pop test will push 5 numbers to the buffer and then pop 6 times\n\n");
    for i in 1..=5{
        josievec.push("element");
        println!("After push {i} JosieVec capacity is {}", josievec.capacity());
        println!("After push {i} JosieVec length is {}", josievec.len());
    }
    for i in 1..=6{
        match josievec.pop(){
            Some(result) => println!("Popped {} on iteration {}", result, i),
            None => println!("Pop Retuned None on iteration {}", i)
        }
    }
    reset_josievec(josievec);
}

fn reserve_and_shrink(josievec:&mut JosieVec<&str>){
    println!("\nReserve and shrink will reserve and check for capacity, making sure returns expected values\n\n");
    josievec.reserve_exact(9);
    println!("After Reserve Exact 9 elements, capacity is now {} and length is now {}", josievec.capacity(), josievec.len());
    josievec.extend([
        "1",
        "2",
        "3"
    ]);
    josievec.shrink_to_fit();
    println!("Wrote 3 elements then shrunk JosieVec, capacity is now {}", josievec.capacity());
    josievec.truncate(2);
    println!("Truncated to length 2, capacity is now {}", josievec.capacity());
    println!("Set length to length 2, capacity is now {}", josievec.capacity());
    josievec.reserve(6);
    println!("After Reserving 6 elements, capacity is now {}", josievec.capacity());
    reset_josievec(josievec);
}
fn slice_and_chunk(josievec:&mut JosieVec<&str>){
    println!("\nChunk and slice will save 10 elements then print them as a slice then chunk them and print that.\n\n");
    josievec.extend(["element";10]);
    println!("Slice is \n{:?}", josievec.as_slice());
    let chunks = josievec.as_chunks::<3>();
    println!("Chunks are\n{:?}", chunks);
    reset_josievec(josievec);
}

pub fn clone_then_check(josievec:&mut JosieVec<&str>){
    println!("\nClone To make sure writing to one doesnt write to the other\n\n");
    let mut josievec_clone = josievec.clone();
    josievec.push("Original");
    josievec_clone.push("Clone");
    println!("Original is {}, Clone is {}",josievec.pop().unwrap(), {josievec_clone.pop().unwrap()});
    reset_josievec(josievec);
}


fn into_boxed_slice_test(){
    println!("\nConverts into a boxed slice and reads to make sure same data is on slice\n\n");
    let mut josievec = JosieVec::with_capacity(4);
    josievec.extend(["Into boxed slice";2]);
    println!("Contents as JosieVec are {:?}, capacity is {}", &josievec[0..2], josievec.capacity());
    let boxed_josievec = josievec.into_boxed_slice();
    println!("Contents as Boxed Slice are {:?}", &boxed_josievec[0..2]);
}

fn mutate_in_place_averaging(max:usize){
    let mut josievec = init_test_vec::<f32>();
    println!("\nGenerates random floats, writes them and averages in place mutating self directly\n\n");
    let mut rng = rand::rng();
    for _ in 0..max{
        josievec.push(rng.random_range(0.0..100.0));
    }
    println!("Original values are {:?}", &josievec[..max]);
    let mut iter = josievec.iter_mut().peekable();
    while let Some(current) = iter.next(){
        if let Some(next) = iter.peek(){
            *current = (*current + **next)/2.0;
        }
    }
    println!("Mutated values are {:?}", josievec.as_slice());
}

pub fn macro_test(){
    println!("Creating a Josievec filled with JosieVecs. tests macro functionality and into_iter\n\n");
    let iterate = josievec!(from repeat("from iter directly").take(2));
    let josievec_of_josievecs = josievec![
        josievec!["with commas", "all of these", "have been ", "macroed"],
        josievec!["elem;num style";3],
        iterate
    ];

    for josievec in josievec_of_josievecs{
        for str in josievec{
            println!("{str}");
        }
    }
}

fn reset_josievec<T>(josievec:&mut JosieVec<T>){
    josievec.clear();
    josievec.shrink_to_fit();
}

fn fibonacci_test(){
    let mut fibonacchi_push:JosieVec<usize> = JosieVec::new();
    fibonacchi_push.fibonacci_push(15);
    fibonacchi_push.fibonacci_push(15);
    println!("original after 2 fibonacci push{:?}", fibonacchi_push.as_slice());
    println!("popped one {:?}", fibonacchi_push.pop());
    println!("removed element 3 {:?}", fibonacchi_push.remove(3));
    println!("remaining elements are {:?}", fibonacchi_push.as_slice());
}

fn mutate_nested_clones(){
    let josievec = josievec!["josievec".to_string()];
    let mut josievec_of_josievec = josievec![josievec.clone(),josievec.clone(),josievec.clone(),josievec.clone(),josievec.clone()];
    for (i,josievec) in josievec_of_josievec.iter_mut().enumerate(){
        josievec.push(format!("number {}",i));
    }
    for josievec in josievec_of_josievec.iter(){
        println!("{:?}", josievec.as_slice())
    }
    drop(josievec);
}

fn drain_then_collect(){
    let mut josievec = josievec![1,2,3,4,5,6,7,8];
    println!("original josievec {:?}", josievec.as_slice());
    //tests both from iterator and drain as the elements are collected into a josievec.
    println!("drained slice is {:?}", josievec.drain(1..5).collect::<JosieVec<i32>>().as_slice());

    println!("after {:?}", josievec.as_slice());
}