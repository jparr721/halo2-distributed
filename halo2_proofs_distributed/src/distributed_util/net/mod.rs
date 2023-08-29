pub fn to_bytes<T>(data: T) -> Vec<u8> {
    let struct_size = std::mem::size_of::<T>();
    let mut bytes = vec![0; struct_size];

    unsafe {
        let struct_ptr = &data as *const T as *const u8;
        std::ptr::copy_nonoverlapping(struct_ptr, bytes.as_mut_ptr(), struct_size);
    }

    bytes
}

pub fn from_bytes<T: Default>(bytes: &[u8]) -> T {
    assert_eq!(bytes.len(), std::mem::size_of::<T>(), "bytes don't match!");

    // Create an empty instance.
    let mut instance = T::default();
    unsafe {
        // Cast the instance over the bytes.
        let struct_ptr = &mut instance as *mut T as *mut u8;

        // Pack the struct.
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), struct_ptr, bytes.len());
    }

    instance
}
