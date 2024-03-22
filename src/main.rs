use std::{ffi::{self, c_void}, ptr, thread};
use colored::Colorize;
use windows::Win32::{self, Graphics::{Gdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BITMAPV4HEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBITMAP, HDC, HPALETTE, RGBQUAD, ROP_CODE, SRCCOPY}, GdiPlus::{GdipCreateBitmapFromHBITMAP, GpBitmap}}, UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::GetDesktopWindow}};

type UINT = libc::c_uint;
type LPVOID = *mut c_void;
type PVOID = *mut c_void;
type LPBITMAPINFO = PVOID;

const NULL: *mut c_void = 0usize as *mut c_void;
const DIB_RGB_COLORS_: UINT = 0;

#[link(name = "gdi32")]
extern "system" {

    fn GetDIBits(hdc: HDC, hbmp: HBITMAP, uStartScan: UINT, cScanLines: UINT,
        lpvBits: LPVOID, lpbi: LPBITMAPINFO, uUsage: UINT) -> libc::c_int;
}

fn main() {
    
    let config: ConfigData = parse_arguments();
    println!("delay: {}, size_x: {}, size_y: {}, type: {}", config.delay, config.rect.size.x, config.rect.size.y, config.scan_type.to_string());

    unsafe
    {let desktop_handle = GetDesktopWindow();
        let h_src = GetDC(desktop_handle);
        let h_dc = CreateCompatibleDC(h_src);
        let h_bitmap = CreateCompatibleBitmap(h_src, config.rect.size.x, config.rect.size.y);
        loop {
            capture_rect(&config, h_src, h_dc, h_bitmap);
        }
    }
}

fn parse_arguments() -> ConfigData
{
    let args: Vec<_> = std::env::args().collect();

    let mut delay: u64 = 100;
    let mut size_w: i32 = 6;
    let mut size_h: i32 = 8;
    let mut scan_type = ScanType::Rect;

    let center_x = 1920 / 2;
    let center_y = 1080 / 2;

    for mut i in 0..args.len() {
        if args[i] == "-d".to_string()
        {
            delay = args[i + 1].to_string().parse::<u64>().unwrap();
            i += 1;
        }
        else if args[i] == "-w".to_string()
        {
            size_w = args[i + 1].to_string().parse::<i32>().unwrap();
            i += 1;
        }
        else if args[i] == "-h".to_string()
        {
            size_h = args[i + 1].to_string().parse::<i32>().unwrap();
            i += 1;
        }
        else if args[i] == "-t".to_string()
        {
            let type_str = args[i + 1].to_string();
            if type_str == "rect".to_string()
            {
                scan_type = ScanType::Rect;
            }
            else if type_str == "circle".to_string()
            {
                scan_type = ScanType::Circle;
            }
            else
            {
                panic!("unknon scan type");    
            }
        }
    }

    return ConfigData::new(delay, size_w, size_h, scan_type, center_x, center_y);
}

unsafe fn capture_rect(config: &ConfigData, h_src: HDC, h_dc: HDC, h_bitmap: HBITMAP)
{
    // let desktop_handle = GetDesktopWindow();
    // let h_src = GetDC(desktop_handle);
    // let h_dc = CreateCompatibleDC(h_src);
    // let h_bitmap = CreateCompatibleBitmap(h_src, config.rect.size.x, config.rect.size.y);
    _ = SelectObject(h_dc, h_bitmap);
    _ = BitBlt(
        h_dc,
        0,
        0,
        config.rect.size.x,
        config.rect.size.y,
        h_src,
        config.rect.pos.x,
        config.rect.pos.y,
        SRCCOPY|CAPTUREBLT
    ).unwrap();
    
    let pixel_width: usize = 4;

    let width = config.rect.size.x;
    let height = config.rect.size.y;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: core::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: height,
            biPlanes: 1,
            biBitCount: 8 * pixel_width as u16,
            biCompression: 0,
            biSizeImage: (width * height * pixel_width as i32) as u32,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,	
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0
        }],
    };
    
    let size: usize = (config.rect.size.x * config.rect.size.y) as usize * 4;
    let mut data: Vec<u8> = Vec::with_capacity(size);
    data.set_len(size);

    GetDIBits(h_dc, h_bitmap, 0, config.rect.size.y as u32,
        &mut data[0] as *mut u8 as *mut c_void,
        &mut bmi as *mut BITMAPINFO as *mut c_void,
        DIB_RGB_COLORS_);

    check_pixels(config, data);

    // DeleteDC(h_dc);
    // ReleaseDC(desktop_handle, h_src);
    // DeleteObject(h_bitmap);
}

pub unsafe fn left_click()
{
    // down left
    let inputs = [ INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 { 
            mi: MOUSEINPUT{
                dx: 0,
                dy: 0,
                mouseData: 0,
                time: 0,
                dwFlags: MOUSEEVENTF_LEFTDOWN,
                dwExtraInfo: 0
            }  
        }
    },
    // release left
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 { 
            mi: MOUSEINPUT{
                dx: 0,
                dy: 0,
                mouseData: 0,
                time: 0,
                dwFlags: MOUSEEVENTF_LEFTUP,
                dwExtraInfo: 0
            }  
        }
    }];
    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
}

unsafe fn check_pixels(config: &ConfigData, data: Vec<u8>)
{
    let lenght = data.len();
			let mut i = 0;
			while i < lenght {
				let r = data[i + 0];
				let g = data[i + 1];
				let b = data[i + 2];
				// PURPLE
				if b >= 180 && r >= 180 && g < 140
				{
                    if GetAsyncKeyState(6) < 0
                    {
                        if GetAsyncKeyState(1) < 0
                        {
                            break;
                        }
                        left_click();
                        std::thread::sleep(std::time::Duration::from_millis(config.delay));
                    }
                    break;
				}
				i += 4;
                // println!("{}", format!("{} {} {}",  r, g, b).truecolor(r, g, b));
			}
}

pub struct ConfigData
{
    delay: u64,
    scan_type: ScanType,
    rect: Rect
}

impl ConfigData
{
    fn new(delay: u64, size_x: i32, size_y: i32,scan_type: ScanType, center_x: i32, center_y: i32) -> Self
    {
        return ConfigData {
            delay: delay,
            scan_type: scan_type,
            rect: Rect { 
                pos: Vec2i::new(center_x - (size_x / 2), center_y - (size_y / 2)),
                size: Vec2i::new(size_x, size_y) 
            }
        }
    }
}

pub struct Rect
{
    pos: Vec2i,
    size: Vec2i
}

pub struct Vec2i
{
    x: i32,
    y: i32
}

impl Vec2i
{
    fn new(x: i32, y: i32) -> Self
    {
        return Vec2i { x: x, y: y };
    }
}

enum ScanType {
    Rect,
    Circle
}

impl ToString for ScanType {
    fn to_string(&self) -> String {
      match self {
        ScanType::Rect => String::from("rect"),
        ScanType::Circle => String::from("circle")
      }
    }
  }