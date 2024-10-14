// turn ugly chafa_sys bindings into nice idiomatic rust structs and functions

use chafa_sys::*;

pub mod Symbols {
    pub const NONE: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_NONE;
    pub const SPACE: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_SPACE;
    pub const SOLID: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_SOLID;
    pub const STIPPLE: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_STIPPLE;
    pub const BLOCK: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_BLOCK;
    pub const BORDER: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_BORDER;
    pub const DIAGONAL: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_DIAGONAL;
    pub const DOT: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_DOT;
    pub const QUAD: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_QUAD;
    pub const HHALF: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_HHALF;
    pub const VHALF: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_VHALF;
    pub const HALF: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_HALF;
    pub const INVERTED: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_INVERTED;
    pub const BRAILLE: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_BRAILLE;
    pub const TECHNICAL: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_TECHNICAL;
    pub const GEOMETRIC: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_GEOMETRIC;
    pub const ASCII: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_ASCII;
    pub const ALPHA: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_ALPHA;
    pub const DIGIT: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_DIGIT;
    pub const ALNUM: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_ALNUM;
    pub const NARROW: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_NARROW;
    pub const WIDE: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_WIDE;
    pub const AMBIGUOUS: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_AMBIGUOUS;
    pub const UGLY: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_UGLY;
    pub const LEGACY: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_LEGACY;
    pub const SEXTANT: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_SEXTANT;
    pub const WEDGE: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_WEDGE;
    pub const LATIN: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_LATIN;
    pub const IMPORTED: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_IMPORTED;
    pub const EXTRA: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_EXTRA;
    pub const BAD: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_BAD;
    pub const ALL: i32 = chafa_sys::ChafaSymbolTags_CHAFA_SYMBOL_TAG_ALL;
}

pub mod PixelType {
    // std::os::raw:c_uint is pretty much u32
    pub const RGBA8_PREMULTIPLIED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_RGBA8_PREMULTIPLIED;
    pub const BGRA8_PREMULTIPLIED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_BGRA8_PREMULTIPLIED;
    pub const ARGB8_PREMULTIPLIED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_ARGB8_PREMULTIPLIED;
    pub const ABGR8_PREMULTIPLIED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_ABGR8_PREMULTIPLIED;
    pub const RGBA8_UNASSOCIATED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_RGBA8_UNASSOCIATED;
    pub const BGRA8_UNASSOCIATED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_BGRA8_UNASSOCIATED;
    pub const ARGB8_UNASSOCIATED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_ARGB8_UNASSOCIATED;
    pub const ABGR8_UNASSOCIATED: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_ABGR8_UNASSOCIATED;
    pub const RGB8: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_RGB8;
    pub const BGR8: u32 = chafa_sys::ChafaPixelType_CHAFA_PIXEL_BGR8;
}

// hide for now
// only necessary for sixel/kitty passthrough, which we won't use for a while or ever
// and this feature is on newer chafa versions
// better to stick with older chafa versions for better compatiblity
//
// pub mod Passthrough {
//     pub const NONE: u32 = chafa_sys::ChafaPassthrough_CHAFA_PASSTHROUGH_NONE;
//     pub const SCREEN: u32 = chafa_sys::ChafaPassthrough_CHAFA_PASSTHROUGH_SCREEN;
//     pub const TMUX: u32 = chafa_sys::ChafaPassthrough_CHAFA_PASSTHROUGH_TMUX;
// }

pub mod CanvasMode {
    pub const TRUECOLOR: u32 = chafa_sys::ChafaCanvasMode_CHAFA_CANVAS_MODE_TRUECOLOR;
    pub const INDEXED_8: u32 = chafa_sys::ChafaCanvasMode_CHAFA_CANVAS_MODE_INDEXED_8;
}

pub type SymbolTagsEnum = i32;
pub type PixelTypeEnum = u32;
// pub type PassthroughEnum = u32;
pub type CanvasModeEnum = u32;

// --- structs holding C pointers and associated functions --- //
// most of the _ptrs are for naive structs that look like { _unused : [u8 ; 0] , }

pub struct SymbolMap {
    _ptr: *mut ChafaSymbolMap,
}

impl SymbolMap {
    pub fn new() -> Self {
        unsafe {
            Self {
                _ptr: chafa_sys::chafa_symbol_map_new(),
            }
        }
    }

    // performance note:
    // "The number of available symbols is a significant factor in the speed...
    // For the fastest possible operation you could use a single symbol --
    // CHAFA_SYMBOL_TAG_VHALF works well by itself."
    //
    // https://hpjansson.org/chafa/ref/chafa-ChafaSymbolMap.html
    pub fn add_by_tags(&self, symbol_tags: SymbolTagsEnum) {
        unsafe {
            chafa_symbol_map_add_by_tags(self._ptr, symbol_tags);
        }
    }

    // TODO:
    // remove_by_tags
    // add_by_range
    // remove_by_range

    // chafa_symbol_map_add_by_range( ..., gunichar first, gunichar last)
    // type gunichar = guint32
    // https://doc.rust-lang.org/std/primitive.char.html
}

impl core::ops::Drop for SymbolMap {
    fn drop(&mut self) {
        unsafe {
            chafa_symbol_map_unref(self._ptr);
        }
    }
}

pub struct Config {
    _ptr: *mut ChafaCanvasConfig,
}

impl Config {
    pub fn new() -> Self {
        unsafe {
            Self {
                _ptr: chafa_sys::chafa_canvas_config_new(),
            }
        }
    }

    pub fn set_geometry(&self, width: i32, height: i32) {
        unsafe {
            chafa_sys::chafa_canvas_config_set_geometry(self._ptr, width, height);
        }
    }

    pub fn set_symbol_map(&self, symbol_map: SymbolMap) {
        unsafe {
            chafa_sys::chafa_canvas_config_set_symbol_map(self._ptr, symbol_map._ptr);
        }
    }

    pub fn set_work_factor(&self, work_factor: f32) {
        // work_factor from 0.0 to 1.0
        unsafe {
            chafa_sys::chafa_canvas_config_set_work_factor(self._ptr, work_factor);
        }
    }

    // pub fn set_passthrough(&self, passthrough: PassthroughEnum) {
    //     unsafe {
    //         chafa_sys::chafa_canvas_config_set_passthrough(self._ptr, passthrough);
    //     }
    // }

    pub fn set_canvas_mode(&self, canvas_mode: CanvasModeEnum) {
        unsafe {
            chafa_sys::chafa_canvas_config_set_passthrough(self._ptr, canvas_mode);
        }
    }

    // TODO:
    // all the other config options
}

impl core::ops::Drop for Config {
    fn drop(&mut self) {
        unsafe {
            chafa_canvas_config_unref(self._ptr);
        }
    }
}

pub struct Canvas {
    _ptr: *mut ChafaCanvas,
}

impl Canvas {
    pub fn new(config: Config) -> Self {
        unsafe {
            Self {
                _ptr: chafa_sys::chafa_canvas_new(config._ptr),
            }
        }
    }

    // "Replaces pixel data of canvas with a copy of that found at src_pixels"
    pub fn draw_all_pixels(
        &self,
        pixel_type: PixelTypeEnum,
        pixels: &[u8],
        width: i32,
        height: i32,
        rowstride: i32,
    ) {
        unsafe {
            chafa_canvas_draw_all_pixels(
                self._ptr,
                pixel_type,
                pixels.as_ptr(),
                width,
                height,
                rowstride,
            );
        }
    }

    pub fn build_ansi(&self) -> String {
        unsafe {
            let gstr: *mut _GString = chafa_canvas_build_ansi(self._ptr);

            // wrapping raw bytes of C strings into friendly Rust types,
            // so output can be manipulated in Rust and passed to println,
            // instead of using glib print helpers like
            // g_print((*result_gstr).str_); or
            // g_printerr((*result_gstr).str_);
            //
            // https://doc.rust-lang.org/std/ffi/struct.CStr.html
            let cstr = std::ffi::CStr::from_ptr((*gstr).str_);
            let result = String::from_utf8_lossy(cstr.to_bytes()).to_string();

            g_string_free(gstr, 1);
            return result;
        }
    }

    // TODO:
    // print(&self, term_info)
    // and other functions
}

impl core::ops::Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            chafa_canvas_unref(self._ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_pixel_output() {
        // just a red block char
        const PIX_WIDTH : i32 = 1;
        const PIX_HEIGHT : i32 = 1;
        const N_CHANNELS : i32 = 4;
        let pixels : [u8; 4] = [0xff, 0x00, 0x00, 0xff];

        let symbol_map = SymbolMap::new();
        symbol_map.add_by_tags(Symbols::BLOCK);

        let config = Config::new();
        config.set_geometry(1, 1);
        config.set_symbol_map(symbol_map);

        let canvas = Canvas::new(config);

        canvas.draw_all_pixels(PixelType::RGBA8_UNASSOCIATED,
                               &pixels,
                               PIX_WIDTH,
                               PIX_HEIGHT,
                               PIX_WIDTH * N_CHANNELS);

        let output : String = canvas.build_ansi();
        assert_eq!(output, "[0m[38;2;254;0;0m█[0m");
    }
}
