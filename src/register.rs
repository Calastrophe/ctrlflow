use serde::Serialize;

/// A trait which is required for each register in the target architecture to implement.
pub trait RegisterInfo
where
    Self: Serialize + Send + Copy + Sized + 'static,
{
    /// Returns a reference to the unique [`Info`] struct for the given register.
    fn info(&self) -> &'static Info<Self>;

    /// Provides a static iterator over the registers in the architecture.
    fn iter() -> std::slice::Iter<'static, Self>;
}

/// All information needed for a single register in the target architecture.
#[derive(Serialize)]
pub struct Info<R>
where
    R: Serialize + Send + Copy,
{
    /// The name which you want to be displayed in interface.
    name: &'static str,
    /// The register's individual enum variant.
    register: R,
    /// The base register of the register's set.
    base: R,
    /// The full register of the register, if it has one.
    full_register: Option<R>,
    /// The size **in bytes** of a register.
    size: u16,
}

impl<R> Info<R>
where
    R: Serialize + Send + Copy,
{
    /// Creates a new instance.
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
