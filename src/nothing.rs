// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;


fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    loop {

    }
}
