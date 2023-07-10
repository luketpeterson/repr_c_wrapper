An opaque `#[repr(C)]` wrapper for `#[repr(Rust)]` types that can be passed by value over FFI

**IMPORTANT** Only types requiring 8 Byte alignment or less can be wrapped, and the C
environment must align uint64_t to at least 8 Byte boundaries.

```
use repr_c_wrapper::*;

#[repr(C)]
pub struct OpaqueWrapper(repr_c_wrapper_t!(String));

#[no_mangle]
pub extern "C" fn some_func() -> OpaqueWrapper {
   OpaqueWrapper("hello".to_string().into())
}
```

*Acknowledgment* Thanks to [@QuineDot](https://github.com/QuineDot), [@h2co3](https://github.com/H2CO3), and 
[@bruecki](https://users.rust-lang.org/u/bruecki/summary) for identifying unsound practices in earlier drafts of this crate.
