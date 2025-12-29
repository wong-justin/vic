// suppress a lot of warnings, and even a couple errors, about C-style naming conventions
//
// there will still be warnings about unsafe FFI things like u128 ints
// and dereferencing null pointers, but it still compiles so that's cool
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!("bindings.rs");

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_some_functions() {
        unsafe {
            println!("{:?}", super::clock());
            println!("{:?}", super::tzset());
            println!("{:?}", super::pthread_testcancel());
            println!("{:?}", super::g_byte_array_new());
            println!("{:?}", super::g_get_user_name());
            println!("{:?}", super::g_get_user_runtime_dir());
            println!("{:?}", super::g_ptr_array_new());
            println!("{:?}", super::__ctype_get_mb_cur_max());
            println!("{:?}", super::chafa_term_info_error_quark());
            println!("{:?}", super::chafa_symbol_map_new());
            println!("{:?}", super::chafa_term_db_new());
            println!("{:?}", super::chafa_term_info_new());
        }
    }

    #[test]
    fn test_chafa_example() {
        unsafe {
            // https://hpjansson.org/chafa/ref/chafa-using.html

            const PIX_WIDTH: i32 = 3;
            const PIX_HEIGHT: i32 = 3;
            const N_CHANNELS: i32 = 4;
            const pixels: [u8; (PIX_WIDTH * PIX_HEIGHT * N_CHANNELS) as usize] = [
                0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0x00, 0x00,
                0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff,
                0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff,
            ];
            let pixels_ptr: *const u8 = pixels.as_ptr();

            let symbol_map = chafa_symbol_map_new();
            chafa_symbol_map_add_by_tags(symbol_map, ChafaSymbolTags_CHAFA_SYMBOL_TAG_ALL);

            let config = chafa_canvas_config_new();
            chafa_canvas_config_set_geometry(config, 23, 6);
            chafa_canvas_config_set_symbol_map(config, symbol_map);

            let canvas = chafa_canvas_new(config);

            chafa_canvas_draw_all_pixels(
                canvas,
                ChafaPixelType_CHAFA_PIXEL_RGBA8_UNASSOCIATED,
                pixels_ptr,
                PIX_WIDTH,
                PIX_HEIGHT,
                PIX_WIDTH * N_CHANNELS,
            );
            // the gstring type, in rust:
            //
            // pub struct _GString {
            //      pub str_ : * mut gchar ,
            //      pub len : gsize ,
            //      pub allocated_len : gsize ,
            // }
            //
            // and type gchar = std::os::raw::c_char

            let gstr_result = chafa_canvas_build_ansi(canvas);
            println!("{}", (*(*gstr_result).str_));

            g_string_free(gstr_result, 1);
            chafa_canvas_unref(canvas);
            chafa_canvas_config_unref(config);
            chafa_symbol_map_unref(symbol_map);
        }
    }
}
