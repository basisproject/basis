pub fn empty_opt<T>(val: &T) -> Option<&T>
    where T: Default + PartialEq
{
    if val == &Default::default() {
        None
    } else {
        Some(val)
    }
}

