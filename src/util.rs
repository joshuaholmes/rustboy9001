//
// Author: Joshua Holmes
//

use std::str;

/// Reads a section of the given vector defined by the range (start..end)
/// into the given array. 
pub fn get_subarray_of_vector(mut arr: &mut [u8], vec: &Vec<u8>, start: usize) {
    let end = start + arr.len();

    if vec.len() < end - 1 {
        panic!("Error! Attempting to read past the end of a vector");
    }

    for (arr_index, vec_index) in (start..end).enumerate() {
        arr[arr_index] = vec[vec_index];
    }
}

/// Converts the given u8 slice into a string
pub fn bytes_to_string(bytes: &[u8]) -> &str {
    match str::from_utf8(bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}