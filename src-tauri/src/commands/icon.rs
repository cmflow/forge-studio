// 提取 exe 的系统图标（SHGetFileInfoW）→ BGRA → PNG (DataURL)
// 结果按 (path, size) 缓存在内存，避免每次列表刷新都重取。
use std::collections::HashMap;
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;

static ICON_CACHE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn cache_get(key: &str) -> Option<String> {
    ICON_CACHE.lock().ok().and_then(|m| m.get(key).cloned())
}

fn cache_put(key: String, value: String) {
    if let Ok(mut m) = ICON_CACHE.lock() {
        m.insert(key, value);
    }
}

/// 返回 `data:image/png;base64,...` DataURL；失败返回 Err（前端降级为纯文字）。
#[tauri::command]
pub fn get_launcher_icon(path: String, size: Option<u16>) -> Result<String, String> {
    let size = size.unwrap_or(32).clamp(16, 256);
    let key = format!("{}#{}", path, size);
    if let Some(v) = cache_get(&key) {
        return Ok(v);
    }
    if !std::path::Path::new(&path).exists() {
        return Err("路径不存在".into());
    }
    let png = extract_png(&path, size)?;
    let data_url = format!("data:image/png;base64,{}", BASE64.encode(&png));
    cache_put(key, data_url.clone());
    Ok(data_url)
}

#[cfg(not(target_os = "windows"))]
fn extract_png(_path: &str, _size: u16) -> Result<Vec<u8>, String> {
    Err("当前平台暂未支持图标提取".into())
}

// -----------------------------
// Windows 平台实现：直接 FFI
// -----------------------------
#[cfg(target_os = "windows")]
mod ffi {
    #![allow(non_camel_case_types, non_snake_case, dead_code)]
    use std::os::raw::{c_int, c_uint, c_void};

    pub type HANDLE = *mut c_void;
    pub type HICON = HANDLE;
    pub type HBITMAP = HANDLE;
    pub type HDC = HANDLE;
    pub type HGDIOBJ = HANDLE;
    pub type BOOL = c_int;
    pub type DWORD = c_uint;
    pub type WORD = u16;
    pub type LONG = i32;
    pub type WCHAR = u16;

    pub const SHGFI_ICON: DWORD = 0x000000100;
    pub const SHGFI_LARGEICON: DWORD = 0x000000000;
    pub const SHGFI_SMALLICON: DWORD = 0x000000001;

    pub const BI_RGB: DWORD = 0;
    pub const DIB_RGB_COLORS: c_uint = 0;

    #[repr(C)]
    pub struct SHFILEINFOW {
        pub hIcon: HICON,
        pub iIcon: c_int,
        pub dwAttributes: DWORD,
        pub szDisplayName: [WCHAR; 260],
        pub szTypeName: [WCHAR; 80],
    }

    #[repr(C)]
    pub struct ICONINFO {
        pub fIcon: BOOL,
        pub xHotspot: DWORD,
        pub yHotspot: DWORD,
        pub hbmMask: HBITMAP,
        pub hbmColor: HBITMAP,
    }

    #[repr(C)]
    pub struct BITMAP {
        pub bmType: LONG,
        pub bmWidth: LONG,
        pub bmHeight: LONG,
        pub bmWidthBytes: LONG,
        pub bmPlanes: WORD,
        pub bmBitsPixel: WORD,
        pub bmBits: *mut c_void,
    }

    #[repr(C)]
    pub struct BITMAPINFOHEADER {
        pub biSize: DWORD,
        pub biWidth: LONG,
        pub biHeight: LONG,
        pub biPlanes: WORD,
        pub biBitCount: WORD,
        pub biCompression: DWORD,
        pub biSizeImage: DWORD,
        pub biXPelsPerMeter: LONG,
        pub biYPelsPerMeter: LONG,
        pub biClrUsed: DWORD,
        pub biClrImportant: DWORD,
    }

    #[repr(C)]
    pub struct RGBQUAD {
        pub rgbBlue: u8,
        pub rgbGreen: u8,
        pub rgbRed: u8,
        pub rgbReserved: u8,
    }

    #[repr(C)]
    pub struct BITMAPINFO {
        pub bmiHeader: BITMAPINFOHEADER,
        pub bmiColors: [RGBQUAD; 1],
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn SHGetFileInfoW(
            pszPath: *const WCHAR,
            dwFileAttributes: DWORD,
            psfi: *mut SHFILEINFOW,
            cbFileInfo: c_uint,
            uFlags: c_uint,
        ) -> usize;
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn DestroyIcon(hIcon: HICON) -> BOOL;
        pub fn GetIconInfo(hIcon: HICON, piconinfo: *mut ICONINFO) -> BOOL;
    }

    #[link(name = "gdi32")]
    extern "system" {
        pub fn GetObjectW(h: HGDIOBJ, c: c_int, pv: *mut c_void) -> c_int;
        pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
        pub fn DeleteDC(hdc: HDC) -> BOOL;
        pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
        pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
        pub fn GetDIBits(
            hdc: HDC,
            hbm: HBITMAP,
            start: c_uint,
            cLines: c_uint,
            lpvBits: *mut c_void,
            lpbmi: *mut BITMAPINFO,
            usage: c_uint,
        ) -> c_int;
    }
}

#[cfg(target_os = "windows")]
fn extract_png(path: &str, size: u16) -> Result<Vec<u8>, String> {
    use ffi::*;
    use std::ptr::null_mut;

    // 路径 → UTF-16 NUL 终止
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    // ---- 拿 HICON
    let flag_size = if size <= 16 { SHGFI_SMALLICON } else { SHGFI_LARGEICON };
    let mut info: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let cb = std::mem::size_of::<SHFILEINFOW>() as u32;
    let ret = unsafe {
        SHGetFileInfoW(
            wide.as_ptr(),
            0,
            &mut info as *mut _,
            cb,
            SHGFI_ICON | flag_size,
        )
    };
    if ret == 0 || info.hIcon.is_null() {
        return Err("SHGetFileInfoW 未返回图标".into());
    }
    let hicon = info.hIcon;

    // ---- 从 HICON 拿 HBITMAP
    let mut icon_info: ICONINFO = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetIconInfo(hicon, &mut icon_info) };
    if ok == 0 {
        unsafe { DestroyIcon(hicon) };
        return Err("GetIconInfo 失败".into());
    }
    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;
    // 注意：hbm_mask 暂不删除，后面若颜色位图无 alpha 需用它重建 alpha
    if hbm_color.is_null() {
        if !hbm_mask.is_null() {
            unsafe { DeleteObject(hbm_mask) };
        }
        unsafe { DestroyIcon(hicon) };
        return Err("图标无颜色位图".into());
    }

    // ---- 读位图尺寸
    let mut bm: BITMAP = unsafe { std::mem::zeroed() };
    let got = unsafe {
        GetObjectW(
            hbm_color,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut _ as *mut _,
        )
    };
    if got == 0 {
        unsafe { DeleteObject(hbm_color) };
        if !hbm_mask.is_null() {
            unsafe { DeleteObject(hbm_mask) };
        }
        unsafe { DestroyIcon(hicon) };
        return Err("GetObjectW 失败".into());
    }
    let w = bm.bmWidth;
    let h = bm.bmHeight.abs();
    if w <= 0 || h <= 0 {
        unsafe { DeleteObject(hbm_color) };
        if !hbm_mask.is_null() {
            unsafe { DeleteObject(hbm_mask) };
        }
        unsafe { DestroyIcon(hicon) };
        return Err("位图尺寸非法".into());
    }

    // ---- GetDIBits 读颜色 BGRA
    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = w;
    bmi.bmiHeader.biHeight = -h; // 负值 = 顶部朝下
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let stride = w * 4; // 32bpp 天然 4 字节对齐
    let mut pixels = vec![0u8; (stride * h) as usize];

    let hdc = unsafe { CreateCompatibleDC(null_mut()) };
    if hdc.is_null() {
        unsafe { DeleteObject(hbm_color) };
        if !hbm_mask.is_null() {
            unsafe { DeleteObject(hbm_mask) };
        }
        unsafe { DestroyIcon(hicon) };
        return Err("CreateCompatibleDC 失败".into());
    }
    let prev = unsafe { SelectObject(hdc, hbm_color) };
    let lines = unsafe {
        GetDIBits(
            hdc,
            hbm_color,
            0,
            h as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };
    unsafe { SelectObject(hdc, prev) };

    if lines == 0 {
        unsafe { DeleteDC(hdc) };
        unsafe { DeleteObject(hbm_color) };
        if !hbm_mask.is_null() {
            unsafe { DeleteObject(hbm_mask) };
        }
        unsafe { DestroyIcon(hicon) };
        return Err("GetDIBits (color) 失败".into());
    }

    // 检查颜色位图是否有有效 alpha（32bpp icon 通常有；24bpp+mask 类图标 alpha 恒 0）
    let mut any_alpha = false;
    for i in 0..(w * h) as usize {
        if pixels[i * 4 + 3] != 0 {
            any_alpha = true;
            break;
        }
    }

    // 若无 alpha：用 mask 位图重建（mask 位 1 = 透明，0 = 不透明）
    if !any_alpha && !hbm_mask.is_null() {
        let mut mask_bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
        mask_bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        mask_bmi.bmiHeader.biWidth = w;
        mask_bmi.bmiHeader.biHeight = -h;
        mask_bmi.bmiHeader.biPlanes = 1;
        mask_bmi.bmiHeader.biBitCount = 32;
        mask_bmi.bmiHeader.biCompression = BI_RGB;

        let mut mask_pixels = vec![0u8; (stride * h) as usize];
        let prev_m = unsafe { SelectObject(hdc, hbm_mask) };
        let mlines = unsafe {
            GetDIBits(
                hdc,
                hbm_mask,
                0,
                h as u32,
                mask_pixels.as_mut_ptr() as *mut _,
                &mut mask_bmi,
                DIB_RGB_COLORS,
            )
        };
        unsafe { SelectObject(hdc, prev_m) };
        if mlines != 0 {
            for i in 0..(w * h) as usize {
                // mask 被 GetDIBits 扩成 32bpp：黑(0)=不透明 白(0xFF)=透明
                let m = mask_pixels[i * 4];
                pixels[i * 4 + 3] = if m == 0 { 0xFF } else { 0x00 };
            }
        } else {
            // mask 读失败，退化为整块不透明，至少避免"全透明看不见"
            for i in 0..(w * h) as usize {
                pixels[i * 4 + 3] = 0xFF;
            }
        }
    }

    unsafe { DeleteDC(hdc) };
    unsafe { DeleteObject(hbm_color) };
    if !hbm_mask.is_null() {
        unsafe { DeleteObject(hbm_mask) };
    }
    unsafe { DestroyIcon(hicon) };

    // ---- BGRA -> RGBA
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        let s = i * 4;
        rgba[s] = pixels[s + 2];
        rgba[s + 1] = pixels[s + 1];
        rgba[s + 2] = pixels[s];
        rgba[s + 3] = pixels[s + 3];
    }

    // ---- PNG 编码
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w as u32, h as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("png header: {}", e))?;
        writer
            .write_image_data(&rgba)
            .map_err(|e| format!("png encode: {}", e))?;
    }
    Ok(out)
}
