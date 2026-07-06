//! [`declare_module!`] — the one line a module writes to be linkable.
//!
//! The generated registry (`omm-modules`) constructs each module by calling a
//! free function named `module()` in the module's crate root. This macro emits
//! that function, so a module author declares an entry point without knowing the
//! registry's calling convention — and the core never learns the module's name.

/// Emit the `module()` entry point the generated registry links against.
///
/// Pass an expression that builds your module — usually `MyModule::default()` or
/// a `MyModule::new()` that seeds state:
///
/// ```
/// use omm_module_api::{declare_module, Module, ModuleManifest, ServerHooks};
///
/// #[derive(Default)]
/// struct MyModule;
/// impl ServerHooks for MyModule {}
/// impl Module for MyModule {
///     fn manifest(&self) -> ModuleManifest {
///         ModuleManifest::new("my-module", env!("CARGO_PKG_VERSION"))
///     }
///     fn as_any(&self) -> &dyn std::any::Any {
///         self
///     }
/// }
///
/// declare_module!(MyModule::default());
///
/// // The macro produced a `module()` returning a boxed `dyn Module`:
/// assert_eq!(module().manifest().name, "my-module");
/// ```
#[macro_export]
macro_rules! declare_module {
    ($ctor:expr) => {
        /// Construct this crate's module — the entry point the generated
        /// `omm-modules` registry calls to link it in. Emitted by
        /// [`omm_module_api::declare_module!`].
        #[must_use]
        pub fn module() -> ::std::boxed::Box<dyn $crate::Module> {
            ::std::boxed::Box::new($ctor)
        }
    };
}
