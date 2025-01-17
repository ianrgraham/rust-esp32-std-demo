use std::ptr::{null, null_mut};

use esp_idf_sys::{
    esp, rmt_config, rmt_config_t, rmt_config_t__bindgen_ty_1, rmt_driver_install,
    rmt_get_counter_clock, rmt_item32_t, rmt_item32_t__bindgen_ty_1,
    rmt_item32_t__bindgen_ty_1__bindgen_ty_1, rmt_mode_t_RMT_MODE_TX, rmt_translator_init,
    rmt_tx_config_t, rmt_wait_tx_done, rmt_write_sample, u_int8_t,
};

pub use rgb::RGB8;
use std::ffi::c_void;

const WS2811_T0H_NS: u32 = 400;
const WS2811_T0L_NS: u32 = 850;
const WS2811_T1H_NS: u32 = 800;
const WS2811_T1L_NS: u32 = 450;

const WS2812_T0H_NS: u32 = 350;
const WS2812_T0L_NS: u32 = 1000;
const WS2812_T1H_NS: u32 = 1000;
const WS2812_T1L_NS: u32 = 350;

#[derive(Debug, Default, Clone, Copy)]
struct Ws2812Config {
    t0h_ticks: u32,
    t0l_ticks: u32,
    t1h_ticks: u32,
    t1l_ticks: u32,
}

const FREERTOS_HZ: u32 = 1000;

static mut WS_CONFIG: Option<Ws2812Config> = None;

unsafe extern "C" fn ws2812_to_rmt(
    src: *const c_void,
    dest: *mut rmt_item32_t,
    src_size: usize,
    wanted_num: usize,
    translated_size: *mut usize,
    item_num: *mut usize,
) {
    if src == null() || dest == null_mut() {
        *translated_size = 0;
        *item_num = 0;
        return;
    }

    let config = WS_CONFIG.unwrap();
    let mut bit0: rmt_item32_t__bindgen_ty_1__bindgen_ty_1 = Default::default();
    bit0.set_duration0(config.t0h_ticks);
    bit0.set_level0(1);
    bit0.set_duration1(config.t0l_ticks);
    bit0.set_level1(0);

    let bit0 = rmt_item32_t {
        __bindgen_anon_1: rmt_item32_t__bindgen_ty_1 {
            __bindgen_anon_1: bit0,
        },
    };

    let mut bit1: rmt_item32_t__bindgen_ty_1__bindgen_ty_1 = Default::default();
    bit1.set_duration0(config.t1h_ticks);
    bit1.set_level0(1);
    bit1.set_duration1(config.t1l_ticks);
    bit1.set_level1(0);

    let bit1 = rmt_item32_t {
        __bindgen_anon_1: rmt_item32_t__bindgen_ty_1 {
            __bindgen_anon_1: bit1,
        },
    };

    let mut size: usize = 0;
    let mut num = 0;

    let mut psrc = src as *const u_int8_t;
    let mut pdest: *mut rmt_item32_t = dest as _;

    while size < src_size && num < wanted_num {
        for i in 0..8 {
            if *psrc & (1 << (7 - i)) != 0 {
                *pdest = bit1;
            } else {
                *pdest = bit0;
            }
            num += 1;
            pdest = pdest.add(1);
        }
        size += 1;
        psrc = psrc.add(1);
    }
    *translated_size = size;
    *item_num = num;
}

pub enum ColorOrder {
    RGB,
    GRB,
}

pub struct ColorBuffer {
    raw_data: Vec<u8>,
    num: usize,
    order: ColorOrder,
}

impl ColorBuffer {

    pub fn new(n: usize, order: ColorOrder) -> Self {
        Self {
            raw_data: vec![0; n*3],
            num: n,
            order,
        }
    }

    fn raw_buffer(&self) -> *const u8 {
        self.raw_data.as_ptr()
    }

    fn raw_len(&self) -> usize {
        self.num * 3
    }

    pub fn len(&self) -> usize {
        self.num
    }

    pub fn fill(&mut self, color: RGB8) {
        for i in 0..self.num {
            match self.order {
                ColorOrder::RGB => {
                    self.raw_data[i*3] = color.r;
                    self.raw_data[i*3+1] = color.g;
                    self.raw_data[i*3+2] = color.b;
                }
                ColorOrder::GRB => {
                    self.raw_data[i*3] = color.g;
                    self.raw_data[i*3+1] = color.r;
                    self.raw_data[i*3+2] = color.b;
                }
            }
        }
    }

    pub fn set(&mut self, index: usize, color: RGB8) {
        match self.order {
            ColorOrder::RGB => {
                self.raw_data[index*3] = color.r;
                self.raw_data[index*3+1] = color.g;
                self.raw_data[index*3+2] = color.b;
            }
            ColorOrder::GRB => {
                self.raw_data[index*3] = color.g;
                self.raw_data[index*3+1] = color.r;
                self.raw_data[index*3+2] = color.b;
            }
        }
    }
}

pub struct WS2812RMT {
    config: rmt_config_t,
}
impl WS2812RMT {
    pub fn new() -> anyhow::Result<Self> {
        Self::new2(8, 0)
    }

    pub fn new2(pin: i32, channel: u32) -> anyhow::Result<Self> {
        let rmt_tx_config = rmt_tx_config_t {
            carrier_freq_hz: 38000,
            carrier_level: 1,
            idle_level: 0,
            carrier_duty_percent: 33,
            loop_count: 1,
            carrier_en: false,
            loop_en: false,
            idle_output_en: true,
        };

        let config = rmt_config_t {
            rmt_mode: rmt_mode_t_RMT_MODE_TX,
            channel: channel,
            gpio_num: pin,
            clk_div: 2,
            mem_block_num: 1,
            flags: 0,
            __bindgen_anon_1: rmt_config_t__bindgen_ty_1 {
                tx_config: rmt_tx_config,
            },
        };

        unsafe {
            esp!(rmt_config(&config))?;
            esp!(rmt_driver_install(config.channel, 0, 0))?;
            let mut rmt_clock = 0u32;
            esp!(rmt_get_counter_clock(config.channel, &mut rmt_clock))?;

            let ratio = rmt_clock as f64 / 1e9;

            WS_CONFIG = Some(Ws2812Config {
                t0h_ticks: (ratio * WS2812_T0H_NS as f64) as _,
                t0l_ticks: (ratio * WS2812_T0L_NS as f64) as _,
                t1h_ticks: (ratio * WS2812_T1H_NS as f64) as _,
                t1l_ticks: (ratio * WS2812_T1L_NS as f64) as _,
            });

            esp!(rmt_translator_init(config.channel, Some(ws2812_to_rmt)))?;
        }

        Ok(Self { config })
    }

    pub fn set_pixel(&mut self, color: RGB8) -> anyhow::Result<()> {
        let timeout_ms = 1;
        unsafe {
            esp!(rmt_write_sample(
                self.config.channel,
                &[color.g, color.r, color.b] as *const u8, // WS2812 expects GRB, not RGB
                3,
                true,
            ))?;
            esp!(rmt_wait_tx_done(
                self.config.channel,
                (timeout_ms as u32 * FREERTOS_HZ) / 1000,
            ))?;
        }

        Ok(())
    }

    pub fn set_pixels(&mut self, colors: &ColorBuffer) -> anyhow::Result<()> {
        let timeout_ms = colors.len();
        unsafe {
            esp!(rmt_write_sample(
                self.config.channel,
                colors.raw_buffer(),
                colors.raw_len(),
                true,
            ))?;
            esp!(rmt_wait_tx_done(
                self.config.channel,
                (timeout_ms as u32 * FREERTOS_HZ) / 1000,
            ))?;
        }

        Ok(())
    }
}


pub struct WS2811RMT {
    config: rmt_config_t,
}
impl WS2811RMT {
    pub fn new() -> anyhow::Result<Self> {
        let rmt_tx_config = rmt_tx_config_t {
            carrier_freq_hz: 38000,
            carrier_level: 1,
            idle_level: 0,
            carrier_duty_percent: 33,
            loop_count: 1,
            carrier_en: false,
            loop_en: false,
            idle_output_en: true,
        };

        let config = rmt_config_t {
            rmt_mode: rmt_mode_t_RMT_MODE_TX,
            channel: 0,
            gpio_num: 8,
            clk_div: 2,
            mem_block_num: 1,
            flags: 0,
            __bindgen_anon_1: rmt_config_t__bindgen_ty_1 {
                tx_config: rmt_tx_config,
            },
        };

        unsafe {
            esp!(rmt_config(&config))?;
            esp!(rmt_driver_install(config.channel, 0, 0))?;
            let mut rmt_clock = 0u32;
            esp!(rmt_get_counter_clock(config.channel, &mut rmt_clock))?;

            let ratio = rmt_clock as f64 / 1e9;

            WS_CONFIG = Some(Ws2812Config {
                t0h_ticks: (ratio * WS2811_T0H_NS as f64) as _,
                t0l_ticks: (ratio * WS2811_T0L_NS as f64) as _,
                t1h_ticks: (ratio * WS2811_T1H_NS as f64) as _,
                t1l_ticks: (ratio * WS2811_T1L_NS as f64) as _,
            });

            esp!(rmt_translator_init(config.channel, Some(ws2812_to_rmt)))?;
        }

        Ok(Self { config })
    }

    pub fn set_pixel(&mut self, color: RGB8) -> anyhow::Result<()> {
        let timeout_ms = 1;
        unsafe {
            esp!(rmt_write_sample(
                self.config.channel,
                &[color.g, color.r, color.b] as *const u8, // WS2812 expects GRB, not RGB
                3,
                true,
            ))?;
            esp!(rmt_wait_tx_done(
                self.config.channel,
                (timeout_ms as u32 * FREERTOS_HZ) / 1000,
            ))?;
        }

        Ok(())
    }

    pub fn set_pixels(&mut self, colors: &ColorBuffer) -> anyhow::Result<()> {
        let timeout_ms = colors.len();
        unsafe {
            esp!(rmt_write_sample(
                self.config.channel,
                colors.raw_buffer(),
                colors.raw_len(),
                true,
            ))?;
            esp!(rmt_wait_tx_done(
                self.config.channel,
                (timeout_ms as u32 * FREERTOS_HZ) / 1000,
            ))?;
        }

        Ok(())
    }
}