pub mod josie_vec;
pub mod tests;
///supports array type notation and from an iterator directly.
/// ```
/// let jv = josievec![1, 2, 3], let jv = josievec![0;4], let jv = josievec!(from iterator)
#[macro_export]
macro_rules! josievec {
        (from $iter:expr) =>{
        {
           $crate::josie_vec::JosieVec::from_iter($iter)
        }
    };
        [$($element:expr),*$(,)?] => {
        {
            $crate::josie_vec::JosieVec::from_iter([$($element),*].into_iter())
        }
    };
    [$element:expr;$length:expr] =>{
        {
            $crate::josie_vec::JosieVec::from_iter([$element;$length].into_iter())
        }
    };
}