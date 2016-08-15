extern crate winapi;
extern crate kernel32;
extern crate user32;
extern crate shell32;
extern crate libc;

use winapi::windef::{HWND, HMENU, HICON, HBRUSH, HBITMAP};
use winapi::winnt::{LPCWSTR};
use winapi::minwindef::{UINT, DWORD, WPARAM, LPARAM, LRESULT, HINSTANCE};
use winapi::winuser::{WNDCLASSW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

fn to_wstring(str : &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect()
}

static mut hwnd : HWND = 0 as HWND;
static mut hmenu: HMENU = 0 as HMENU;

struct WinApp {
}

pub unsafe extern "system" fn window_proc(h_wnd :HWND,
	                                        msg :UINT,
                                          w_param :WPARAM,
                                          l_param :LPARAM) -> LRESULT
{
    if msg == winapi::winuser::WM_USER + 1 {
        println!("Got callback message! {}", l_param);
        // if l_param as UINT == winapi::winuser::WM_LBUTTONUP {
        //     println!("Posting quit message! {}", l_param);
        //     user32::PostQuitMessage(0);
        // }
        if l_param as UINT == winapi::winuser::WM_LBUTTONUP  {
            let mut p = winapi::POINT {
                x: 0,
                y: 0
            };
            if user32::GetCursorPos(&mut p as *mut winapi::POINT) == 0 {
                println!("Wrong position!");
                return 1;
            }
            println!("showing menu!");
            user32::SetForegroundWindow(hwnd);
            user32::TrackPopupMenu(hmenu, 0, p.x, p.y, (winapi::TPM_BOTTOMALIGN | winapi::TPM_LEFTALIGN) as i32, hwnd, std::ptr::null_mut());
        }
    }
    if msg == winapi::winuser::WM_DESTROY {
        user32::PostQuitMessage(0);
    }
    return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
}

fn main() {
    // Create window

    let class_name = to_wstring("my_window");
    let instance;
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
            hIcon: user32::LoadIconW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
            hCursor: user32::LoadCursorW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
            hbrBackground: 16 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: class_name.as_ptr(),
        };
    }
    unsafe {
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
    }

    // Create and add icon

    let icon;
    unsafe {
        icon = user32::LoadImageA(instance,
                                  std::mem::transmute(0x1 as isize),
                                  winapi::IMAGE_ICON,
                                  64,
                                  64,
                                  0) as HICON;
    }
    println!("Got icon! {:?}", icon as u32);
    let a : [i8; 64] = [0; 64];
    let mut b : [i8; 128] = [0; 128];
    let g : [i8; 256] = [0; 256];
    let guid = winapi::GUID {
        Data1: 0 as winapi::c_ulong,
        Data2: 0 as winapi::c_ushort,
        Data3: 0 as winapi::c_ushort,
        Data4: [0; 8]
    };
    unsafe {
        let mut nid_add = winapi::shellapi::NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<winapi::shellapi::NOTIFYICONDATAA>() as DWORD,
            hWnd: hwnd,
            uID: 0x1 as UINT,
            uFlags: winapi::NIF_MESSAGE,
            uCallbackMessage: winapi::WM_USER + 1,
            hIcon: 0 as HICON,
            szTip: b,
            dwState: 0 as DWORD,
            dwStateMask: 0 as DWORD,
            szInfo: g,
            uTimeout: 0 as UINT,
            szInfoTitle: a,
            dwInfoFlags: 0 as UINT,
            guidItem: guid,
            hBalloonIcon: 0 as HICON
        };
        println!("Adding icon! {}", shell32::Shell_NotifyIconA(winapi::NIM_ADD,
                                                               &mut nid_add as *mut winapi::shellapi::NOTIFYICONDATAA));
    }
    unsafe {
        let mut nid = winapi::shellapi::NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<winapi::shellapi::NOTIFYICONDATAA>() as DWORD,
            hWnd: hwnd,
            uID: 0x1 as UINT,
            uFlags: winapi::NIF_ICON,
            uCallbackMessage: 0 as UINT,
            hIcon: icon,
            szTip: b,
            dwState: 0 as DWORD,
            dwStateMask: 0 as DWORD,
            szInfo: g,
            uTimeout: 0 as UINT,
            szInfoTitle: a,
            dwInfoFlags: 0 as UINT,
            guidItem: guid,
            hBalloonIcon: 0 as HICON
        };
        println!("Setting icon! {}", shell32::Shell_NotifyIconA(winapi::NIM_MODIFY,
                                                                &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));
    }

    // Add Tooltip
    // Gross way to convert String to [i8; 128]
    // TODO: Clean up conversion, test for length so we don't panic at runtime
    let t = "Test Tip".to_string();
    let tt = t.as_bytes();
    for i in 0..tt.len() {
        b[i] = tt[i].clone() as i8;
    }
    unsafe {
        let mut nid_tip = winapi::shellapi::NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<winapi::shellapi::NOTIFYICONDATAA>() as DWORD,
            hWnd: hwnd,
            uID: 0x1 as UINT,
            uFlags: winapi::NIF_TIP,
            uCallbackMessage: 0 as UINT,
            hIcon: 0 as HICON,
            szTip: b,
            dwState: 0 as DWORD,
            dwStateMask: 0 as DWORD,
            szInfo: g,
            uTimeout: 0 as UINT,
            szInfoTitle: a,
            dwInfoFlags: 0 as UINT,
            guidItem: guid,
            hBalloonIcon: 0 as HICON
        };
        println!("Setting tip! {}", shell32::Shell_NotifyIconA(winapi::NIM_MODIFY,
                                                               &mut nid_tip as *mut winapi::shellapi::NOTIFYICONDATAA));
    }

    // Add Menu
    let m;
    unsafe {
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

    // Add Menu Item
    unsafe {
        let idx : u32 = 1;
        let mut st = to_wstring("Quit");
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
            cch: 10,
            hbmpItem: 0 as HBITMAP
        };
        user32::InsertMenuItemW(hmenu, 0, 1, &item as *const winapi::MENUITEMINFOW);
    }

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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
