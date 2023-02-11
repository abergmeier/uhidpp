use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum Type {
    Input = 1,
    Output,
    Feature,
}
