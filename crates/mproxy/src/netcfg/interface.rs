pub type IfIndex = libc::c_int;

pub trait AsIfIndex {
    fn as_if_index(&self) -> IfIndex;
}

impl AsIfIndex for IfIndex {
    fn as_if_index(&self) -> IfIndex {
        *self
    }
}
