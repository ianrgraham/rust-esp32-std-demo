
use esp_idf_sys as _;

// 96.5 inch diameter vertical
// 48 inch half-diameter horizontal

// scan line n on semicircle
// [2, 6, 8, 9, 11, 12, 12, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 14, 14, 13, 12, 12, 11, 9, 8, 6, 2]

// dimensions: 32x32

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let scan_lines = [2u8, 6, 8, 9, 11, 12, 12, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 14, 14, 13, 12, 12, 11, 9, 8, 6, 2];
    loop {

    }
}