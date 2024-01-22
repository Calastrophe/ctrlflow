use serde::Serialize;

pub trait RegisterInfo
where
    Self: Serialize + Send + Copy + Sized + 'static,
{
    /// Returns a reference to the unique [`Info`] struct for the given register.
    fn info(&self) -> &'static Info<Self>;

    /// Provides a static iterator over the registers in the architecture.
    fn iter() -> std::slice::Iter<'static, Self>;
}

#[derive(Serialize)]
pub struct Info<R>
where
    R: Serialize + Send + Copy,
{
    name: &'static str,
    register: R,
    base: R,
    full_register: Option<R>,
    size: u16,
}

impl<R> Info<R>
where
    R: Serialize + Send + Copy,
{
    pub const fn new(
        name: &'static str,
        register: R,
        base: R,
        full_register: Option<R>,
        size: u16,
    ) -> Self {
        Self {
            name,
            register,
            base,
            full_register,
            size,
        }
    }
}
