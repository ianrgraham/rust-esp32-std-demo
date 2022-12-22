
use esp_idf_sys as _;

// this will be easy, just some simple waterfall effect

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    loop {

    }
}