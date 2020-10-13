pub mod core;
pub mod graphics;

#[no_mangle]
pub fn build_platform() -> core::Platform {
   core::Platform::start().unwrap()
}
