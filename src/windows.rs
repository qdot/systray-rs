extern crate winapi;
extern crate kernel32;
extern crate user32;
extern crate shell32;
extern crate libc;

use std;
use winapi::windef::{HWND, HMENU, HICON, HBRUSH, HBITMAP};
use winapi::winnt::{LPCWSTR};
use winapi::minwindef::{UINT, DWORD, WPARAM, LPARAM, LRESULT, HINSTANCE};
use winapi::winuser::{WNDCLASSW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

fn to_wstring(str : &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect()
}

static mut instance : HINSTANCE = 0 as HINSTANCE;
static mut hwnd : HWND = 0 as HWND;
static mut hmenu: HMENU = 0 as HMENU;

pub unsafe extern "system" fn window_proc(h_wnd :HWND,
	                                        msg :UINT,
                                          w_param :WPARAM,
                                          l_param :LPARAM) -> LRESULT
{
    if msg == winapi::winuser::WM_USER + 1 {
        if l_param as UINT == winapi::winuser::WM_LBUTTONUP  {
            let mut p = winapi::POINT {
                x: 0,
                y: 0
            };
            if user32::GetCursorPos(&mut p as *mut winapi::POINT) == 0 {
                return 1;
            }
            user32::SetForegroundWindow(hwnd);
            user32::TrackPopupMenu(hmenu,
                                   0,
                                   p.x,
                                   p.y,
                                   (winapi::TPM_BOTTOMALIGN | winapi::TPM_LEFTALIGN) as i32,
                                   hwnd,
                                   std::ptr::null_mut());
        }
    }
    if msg == winapi::winuser::WM_DESTROY {
        user32::PostQuitMessage(0);
    }
    return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
}

fn get_nid_struct() -> winapi::shellapi::NOTIFYICONDATAA {
    unsafe {
        winapi::shellapi::NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<winapi::shellapi::NOTIFYICONDATAA>() as DWORD,
            hWnd: hwnd,
            uID: 0x1 as UINT,
            uFlags: 0 as UINT,
            uCallbackMessage: 0 as UINT,
            hIcon: 0 as HICON,
            szTip: [0 as i8; 128],
            dwState: 0 as DWORD,
            dwStateMask: 0 as DWORD,
            szInfo: [0 as i8; 256],
            uTimeout: 0 as UINT,
            szInfoTitle: [0 as i8; 64],
            dwInfoFlags: 0 as UINT,
            guidItem: winapi::GUID {
                Data1: 0 as winapi::c_ulong,
                Data2: 0 as winapi::c_ushort,
                Data3: 0 as winapi::c_ushort,
                Data4: [0; 8]
            },
            hBalloonIcon: 0 as HICON
        }
    }
}

pub fn create_window() {
    let class_name = to_wstring("my_window");
    unsafe {
        instance = kernel32::GetModuleHandleA(std::ptr::null_mut());
    };
    let wnd;
    unsafe {
        wnd = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: user32::LoadIconW(0 as HINSTANCE,
                                     winapi::winuser::IDI_APPLICATION),
            hCursor: user32::LoadCursorW(0 as HINSTANCE,
                                         winapi::winuser::IDI_APPLICATION),
            hbrBackground: 16 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: class_name.as_ptr(),
        };
        user32::RegisterClassW(&wnd);
        hwnd = user32::CreateWindowExW(0,
                                       class_name.as_ptr(),
                                       to_wstring("rust_systray_window").as_ptr(),
                                       WS_OVERLAPPEDWINDOW,
                                       CW_USEDEFAULT,
                                       0,
                                       CW_USEDEFAULT,
                                       0,
                                       0 as HWND,
                                       0 as HMENU,
                                       0 as HINSTANCE,
                                       std::ptr::null_mut());
        println!("Got window! {:?}", hwnd as u32);
        println!("Error? {}", kernel32::GetLastError());
        let mut nid = get_nid_struct();
        nid.uID = 0x1;
        nid.uFlags = winapi::NIF_MESSAGE;
        nid.uCallbackMessage = winapi::WM_USER + 1;
        println!("Adding icon! {}", shell32::Shell_NotifyIconA(winapi::NIM_ADD,
                                                               &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));
        // Setup menu
        let m;
        hmenu = user32::CreatePopupMenu();
        m = winapi::MENUINFO {
            cbSize: std::mem::size_of::<winapi::MENUINFO>() as DWORD,
            fMask: winapi::MIM_APPLYTOSUBMENUS | winapi::MIM_STYLE,
            dwStyle: winapi::MNS_NOTIFYBYPOS,
            cyMax: 0 as UINT,
            hbrBack: 0 as HBRUSH,
            dwContextHelpID: 0 as DWORD,
            dwMenuData: 0 as winapi::ULONG_PTR
        };
        println!("Created menu! {}", user32::SetMenuInfo(hmenu, &m as *const winapi::MENUINFO));
    }
}

fn set_icon(icon: HICON) {
    unsafe {
        let mut nid = get_nid_struct();
        nid.uFlags = winapi::NIF_ICON;
        nid.hIcon = icon;
        println!("Setting icon! {}", shell32::Shell_NotifyIconA(winapi::NIM_MODIFY,
                                                                &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));

    }
}

pub fn set_icon_from_resource() {
    let icon;
    unsafe {
        icon = user32::LoadImageA(instance,
                                  std::mem::transmute(0x1 as isize),
                                  winapi::IMAGE_ICON,
                                  64,
                                  64,
                                  0) as HICON;
    }
    set_icon(icon);
}

pub fn set_icon_from_file() {
}

pub fn set_tooltip(tooltip: &String) {
    // Add Tooltip
    // Gross way to convert String to [i8; 128]
    // TODO: Clean up conversion, test for length so we don't panic at runtime
    let tt = tooltip.as_bytes().clone();
    let mut nid = get_nid_struct();
    for i in 0..tt.len() {
        nid.szTip[i] = tt[i] as i8;
    }
    nid.uFlags = winapi::NIF_TIP;
    unsafe {
        println!("Setting tip! {}", shell32::Shell_NotifyIconA(winapi::NIM_MODIFY,
                                                               &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));
    }
}

pub fn add_menu_item(item_name: &String) {
    let idx : u32 = 1;
    let mut st = to_wstring(item_name);
    let item = winapi::MENUITEMINFOW {
        cbSize: std::mem::size_of::<winapi::MENUITEMINFOW>() as UINT,
        fMask: (winapi::MIIM_FTYPE | winapi::MIIM_STRING | winapi::MIIM_DATA | winapi::MIIM_STATE),
        fType: winapi::MFT_STRING,
        fState: 0 as UINT,
        wID: 0 as UINT,
        hSubMenu: 0 as HMENU,
        hbmpChecked: 0 as HBITMAP,
        hbmpUnchecked: 0 as HBITMAP,
        dwItemData: idx as u64,
        dwTypeData: st.as_mut_ptr(),
        cch: (item_name.len() * 2) as u32, // 16 bit characters
        hbmpItem: 0 as HBITMAP
    };
    unsafe {
        user32::InsertMenuItemW(hmenu, 0, 1, &item as *const winapi::MENUITEMINFOW);
    }
}

pub fn run_loop() {
    // Run message loop
    let mut msg = winapi::winuser::MSG {
        hwnd: 0 as HWND,
        message: 0 as UINT,
        wParam: 0 as WPARAM,
        lParam: 0 as LPARAM,
        time: 0 as DWORD,
        pt: winapi::windef::POINT { x: 0, y: 0, },
    };
    loop {
        unsafe {
            user32::GetMessageW(&mut msg, 0 as HWND, 0, 0);
        }
        if msg.message == winapi::winuser::WM_QUIT {
            break;
        }
        unsafe {
            user32::TranslateMessage(&mut msg);
            user32::DispatchMessageW(&mut msg);
        }
    }
}
