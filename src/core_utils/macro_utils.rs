/// Appends additional method to struct implementation with the same name as param.
///
/// # Examples
///
/// ```ignore
/// use rust_simple_chat::self_method;
///
/// struct Foo;
///
/// impl Foo {
///     self_method!(name, String);
/// }
/// ```
#[macro_export]
macro_rules! self_method {
    ($name:ident, $ty:ty) => {
        pub fn $name(mut self, $name: $ty) -> Self {
            self.$name = Some($name);
            self
        }
    };
}
