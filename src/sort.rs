use core::cmp::Ordering;

/// Do a [binary search](https://en.wikipedia.org/wiki/Binary_search_algorithm) looking for a
/// specific `key`, and returning `not_found_value` if not found. ^^
/// 
/// # Arguments
/// 
/// * `key` - Value to search for
/// * `data_ptr` - Constant raw pointer to the array. You can get it using 'data_array.as_ptr()'
/// * `data_length` - Array size. It can be lower than the array capacity.
/// * `not_found_value` - Value to return if value not found
/// * `compare_f` - Function used to compare^^
/// 
/// # Notes
/// ^
/// ^^ This `not_found_value` will limit the possible lenght by -1 element. For example, if
/// the platform is based on 32bits (like wasm), the recommended value for this is the max
/// value of u32 (usize): 0xFFFF_FFFF
/// 
/// # Example
/// 
/// ```
/// ```
/// 
pub fn bsearch<T1, T2, F>(
    key: T1
    , data_ptr: *const T2
    , data_length: usize
    , not_found_value: usize
    , compare_f: F
) -> usize
where F: Fn(&T1, *const T2, usize) -> Ordering, 
{
    let mut left = 0;
    let mut s = data_length;
    while s > 0 {
        let idx = left + (s>>1);
        let r: Ordering = compare_f(&key, data_ptr, idx);
        if r == Ordering::Equal {
            return idx;
        }
        if r == Ordering::Greater {
            left = idx + 1;
            s = s - 1;
        }
        s = s >> 1;
    }
    return not_found_value;
}

/// An `AproxBinarySearchResult` is how the `aprox_bsearch` ended:
/// * `ExactMatchIndex` - The value was found in the array
/// * `AproxMatch` - An index was found inside the current array
/// * `OutsideIndex` - The value should be inserted at the end of the array
pub enum AproxBinarySearchResult {
    ExactMatchIndex
    , AproxMatch
    , OutsideIndex
}

/// Do an aproximate binary search, returning the index of the value
/// or the index where the value should be.^
/// 
/// # Arguments
/// 
/// * `key` - Value to search for
/// * `data_ptr` - Constant raw pointer to the array. You can get it using 'data_array.as_ptr()'
/// * `data_length` - Array size. It can be lower than the array capacity.
/// * `compare_f` - Function used to compare^^
/// 
/// # Notes
/// ^ Since the objective of this algorithms is to be used on embedded systems, these will
///   use raw pointers and not fat pointers (to avoid additional overhead of panic functions),
///   but this requires additional caution from the programer to ensure that the pointers are
///   correct!
/// ^^ The use of this function is to compare a key vs a complex type that contains a key. For
///    example:
/// 
///    struct TestStruct {
///      inside_key: u8
///    }
/// 
/// # Example
/// 
/// Given a sorted array \[0x10, 0x20], we will search for the
/// index of the value 0x15. It should be after 0x10 (index=1)
/// 
/// ```
/// use core::cmp::Ordering;
/// use rselib::sort::{aprox_bsearch, AproxBinarySearchResult};
/// 
/// let test_array: [u8; 2] = [0x10, 0x20];
/// let test_array_ptr = test_array.as_ptr() as *const u8;
/// let length = test_array.len();
/// let (res, possible_index) = aprox_bsearch(
///     0x15
///     , test_array_ptr
///     , length
///     , |key, ptr, index| {
///         let current_value = unsafe {
///             & *(
///                 ptr.add(index)
///             )
///         };
///         if *key == *current_value {
///             return Ordering::Equal
///         } else if *key > *current_value {
///             return Ordering::Greater;
///         } else {
///             return Ordering::Less;
///         }
///     }
/// );
/// assert!(matches!(res, AproxBinarySearchResult::AproxMatch));
/// assert_eq!(1, possible_index);
/// ```
pub fn aprox_bsearch<T1, T2, F>(
    key: T1
    , data_ptr: *const T2
    , data_length: usize
    , compare_f: F
) -> (AproxBinarySearchResult, usize)
where F: Fn(&T1, *const T2, usize) -> Ordering, 
{
    let mut left = 0;
    let mut s = data_length;
    let mut exact_value_found = false;
    let mut idx: usize = 0;
    while (s > 0) & (exact_value_found == false) {
        idx = left + (s>>1);
        let r: Ordering = compare_f(&key, data_ptr, idx);
        if r == Ordering::Equal {
            exact_value_found = true;
            break;
        }
        if r == Ordering::Greater {
            left = idx + 1;
            s = s - 1;
        }
        s = s >> 1;
    }
    if exact_value_found == true {
        return (AproxBinarySearchResult::ExactMatchIndex, idx);
    }
    else {
        if idx >= data_length {
            return (AproxBinarySearchResult::OutsideIndex, idx);
        }
        else {
            match compare_f(&key, data_ptr, idx) {
                Ordering::Less => return (AproxBinarySearchResult::AproxMatch, idx),
                Ordering::Equal => return (AproxBinarySearchResult::ExactMatchIndex, idx),
                Ordering::Greater => return (AproxBinarySearchResult::AproxMatch, idx + 1),
            }
        }
    } 
}

#[derive(Debug)]
pub enum SortedArrayAllocResult {
    Ok
    , ArrayCapacityExceeded
}

/// For a pre allocated array with capacity C, the sorted_array_insert
/// will shift the contents to the right if the value doesnt exist in
/// it, returning the index for the new value. Or only return the index
/// for the existing value.
/// 
/// # Arguments
/// 
/// * `key` - Value to search for
/// * `data_ptr` - Constant raw pointer to the array. You can get it using 'data_array.as_ptr()'
/// * `data_original_length` - Array size. It can be lower than the array capacity
/// * `data_capacity` - Real allocated array length.
/// * `not_allocated_value` - Value to return if the capacity was exceeded
/// * `compare_f` - Function used to compare^^
/// * `copy_f` - Function to shift elements along the array
/// 
/// # Example
/// ```
/// use core::cmp::Ordering;
/// use rselib::sort::{sorted_array_insert, SortedArrayAllocResult};
/// 
/// let mut test_array: [u8; 3] = [0x10, 0x20, 0x00]; // pre allocated array
/// let new_value: u8 = 0x15;
/// let not_allocated_value: usize = 0xFFFF_FFFF;
/// 
/// let test_array_ptr = test_array.as_ptr() as *mut u8;
/// let capacity = test_array.len();
/// let mut length = 2; // Used elements in the pre allocated array
/// 
/// 
/// let (res, possible_index, added_count) = sorted_array_insert(
///     new_value
///     , test_array_ptr
///     , length
///     , capacity
///     , not_allocated_value
///     , |key, ptr, index| {
///         let current_value = unsafe {
///             & *(
///                 ptr.add(index)
///             )
///         };
///         if *key == *current_value {
///             return Ordering::Equal
///         } else if *key > *current_value {
///             return Ordering::Greater;
///         } else {
///             return Ordering::Less;
///         }
///     }, |ptr, src_index, dest_index | {
///         let src = unsafe {
///             & *(
///                 ptr.add(src_index)
///             )
///         };
///         let dest = unsafe {
///             &mut *(
///                 ptr.add(dest_index)
///             )
///         };
///         *dest = *src;
///     }
/// );
/// assert!(matches!(res, SortedArrayAllocResult::Ok));
/// assert_eq!(1, possible_index);
/// assert_eq!(1, added_count);
/// 
/// // Update value
/// test_array[possible_index] = new_value;
/// // !Update length
/// length += added_count;
/// 
/// assert_eq!(0x10, test_array[0]);
/// assert_eq!(new_value, test_array[1]);
/// assert_eq!(0x20, test_array[2]);
/// 
/// assert_eq!(3, length);
/// ```
pub fn sorted_array_insert<T1, T2, F1, F2>(
    key: T1
    , data_ptr: *mut T2
    , data_original_length: usize
    , data_capacity: usize
    , not_allocated_value: usize
    , compare_f: F1
    , copy_f: F2
) -> (SortedArrayAllocResult, usize, usize)
where F1: Fn(&T1, *const T2, usize) -> Ordering, 
F2: Fn(*mut T2, usize, usize)
{
    let (aprox_result, possible_index) = aprox_bsearch(
        key
        , data_ptr
        , data_original_length
        , compare_f
    );
    match aprox_result {
        AproxBinarySearchResult::ExactMatchIndex => {
            return (SortedArrayAllocResult::Ok, possible_index, 0);
        },
        AproxBinarySearchResult::AproxMatch => {
            if data_original_length + 1 <= data_capacity {
                let mut count = data_original_length - possible_index;
                if count > 0 {
                    let mut target_index = data_original_length - 1;
                    loop {
                        copy_f(
                            data_ptr
                            , target_index
                            , target_index + 1
                        );
                        count -= 1;
                        if count == 0 {
                            break;
                        }
                        target_index -= 1;
                    }
                }
                return (SortedArrayAllocResult::Ok, possible_index, 1);
            }
            else {
                return (SortedArrayAllocResult::ArrayCapacityExceeded, not_allocated_value, 0);
            }
        },
        AproxBinarySearchResult::OutsideIndex => {
            if data_original_length + 1 <= data_capacity {
                return (SortedArrayAllocResult::Ok, possible_index, 1);
            }
            else {
                return (SortedArrayAllocResult::ArrayCapacityExceeded, not_allocated_value, 0);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;

    /// Function to compare u8's used in aprox_bsearch
    fn u8_cmp(
        key: &u8
        , ptr: *const u8
        , index: usize
    ) -> Ordering {
        let current_value = unsafe {
            & *(
                ptr.add(index)
            )
        };

        if *key == *current_value {
            return Ordering::Equal
        } else if *key > *current_value {
            return Ordering::Greater;
        } else {
            return Ordering::Less;
        }
    }

    fn u8_cp(
        ptr: *mut u8
        , src_index: usize
        , dest_index: usize
    ) {
        let src = unsafe {
            & *(
                ptr.add(src_index)
            )
        };
        let dest = unsafe {
            &mut *(
                ptr.add(dest_index)
            )
        };
        *dest = *src;
    }

    #[test]
    fn insert_at_the_end() {
        let mut test_array: [u8; 3] = [0x10, 0x20, 0x00]; // pre allocated array
        let new_value: u8 = 0x25;
        let not_allocated_value: usize = 0xFFFF_FFFF;

        let test_array_ptr = test_array.as_ptr() as *mut u8;
        let capacity = test_array.len();
        let mut length = 2; // Used elements in the array
        let (res, possible_index, added_count) = sorted_array_insert(
            new_value
            , test_array_ptr
            , length
            , capacity
            , not_allocated_value
            , u8_cmp
            , u8_cp
        );

        assert!(matches!(res, SortedArrayAllocResult::Ok));
        assert_eq!(possible_index, 2);
        assert_eq!(1, added_count);

        // Write to the pointed
        test_array[possible_index] = new_value;
        // !!Dont forget to update the used length
        length += added_count;

        assert_eq!(0x10, test_array[0]);
        assert_eq!(0x20, test_array[1]);
        assert_eq!(0x25, test_array[2]);
        assert_eq!(3, length);
        
    }

    #[test]
    fn insert_at_the_beginning() {
        let mut test_array: [u8; 3] = [0x10, 0x20, 0x00]; // pre allocated array
        let new_value: u8 = 0x05;
        let not_allocated_value: usize = 0xFFFF_FFFF;

        let test_array_ptr = test_array.as_ptr() as *mut u8;
        let capacity = test_array.len();
        let mut length = 2; // Used elements in the array
        let (res, possible_index, added_count) = sorted_array_insert(
            new_value
            , test_array_ptr
            , length
            , capacity
            , not_allocated_value
            , u8_cmp
            , u8_cp
        );

        assert!(matches!(res, SortedArrayAllocResult::Ok));
        assert_eq!(possible_index, 0);
        assert_eq!(1, added_count);

        // Write to the pointed
        test_array[possible_index] = new_value;
        // !!Dont forget to update the used length
        length += added_count;

        assert_eq!(0x05, test_array[0]);
        assert_eq!(0x10, test_array[1]);
        assert_eq!(0x20, test_array[2]);
        assert_eq!(3, length);
    }

    #[test]
    fn point_to_the_existing_value() {
        let test_array: [u8; 3] = [0x10, 0x20, 0x00]; // pre allocated array
        let new_value: u8 = 0x10;
        let not_allocated_value: usize = 0xFFFF_FFFF;

        let test_array_ptr = test_array.as_ptr() as *mut u8;
        let capacity = test_array.len();
        let length = 2; // Used elements in the array
        let (res, possible_index, added_count) = sorted_array_insert(
            new_value
            , test_array_ptr
            , length
            , capacity
            , not_allocated_value
            , u8_cmp
            , u8_cp
        );

        assert!(matches!(res, SortedArrayAllocResult::Ok));
        assert_eq!(possible_index, 0);

        assert_eq!(0x10, test_array[0]);
        assert_eq!(0x20, test_array[1]);
        assert_eq!(0x00, test_array[2]);
        assert_eq!(0, added_count);
    }

    #[test]
    fn max_capacity_exceeded() {
        let test_array: [u8; 2] = [0x10, 0x20]; // pre allocated array
        let new_value: u8 = 0x15;
        let not_allocated_value: usize = 0xFFFF_FFFF;

        let test_array_ptr = test_array.as_ptr() as *mut u8;
        let capacity = test_array.len();
        let length = 2; // Used elements in the array
        let (res, possible_index, added_count) = sorted_array_insert(
            new_value
            , test_array_ptr
            , length
            , capacity
            , not_allocated_value
            , u8_cmp
            , u8_cp
        );

        assert!(matches!(res, SortedArrayAllocResult::ArrayCapacityExceeded));
        assert_eq!(possible_index, not_allocated_value);

        assert_eq!(0x10, test_array[0]);
        assert_eq!(0x20, test_array[1]);
        assert_eq!(0, added_count);
    }
}