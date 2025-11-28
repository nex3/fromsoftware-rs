/// An opaque, still-unreversed native structure. This is equivalent to a `u8`
/// array, but its use indicates that we know a structure exists and we know its
/// size but we don't know its purpose or anything else about it.
///
/// Don't use this for values that you don't know are distinct structs; just use
/// `[u8; N]` directly instead.
#[repr(C)]
pub struct UnknownStruct<const N: usize>([u8; N]);
