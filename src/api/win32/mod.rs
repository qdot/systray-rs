use {SystrayEvent, SystrayError, Callback, make_callback};
use std;
use std::cell::RefCell;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::thread;
use std::collections::HashMap;
use winapi;
use user32;
use kernel32;
use shell32;
use winapi::windef::{HWND, HMENU, HICON, HBRUSH, HBITMAP};
use winapi::winnt::{LPCWSTR};
use winapi::minwindef::{UINT, DWORD, WPARAM, LPARAM, LRESULT, HINSTANCE};
use winapi::winuser::{WNDCLASSW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT};

fn to_wstring(str : &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}

// Got this idea from glutin. Yay open source! Boo stupid winproc! Even more boo
// doing SetLongPtr tho.
thread_local!(static WININFO_STASH: RefCell<Option<WindowsLoopData>> = RefCell::new(None));

#[derive(Clone)]
struct WindowInfo {
    pub hwnd: HWND,
    pub hinstance: HINSTANCE,
    pub hmenu: HMENU,
}

unsafe impl Send for WindowInfo {}
unsafe impl Sync for WindowInfo {}

#[derive(Clone)]
struct WindowsLoopData {
    pub info: WindowInfo,
    pub tx: Sender<SystrayEvent>
}

unsafe extern "system" fn window_proc(h_wnd :HWND,
	                                    msg :UINT,
                                      w_param :WPARAM,
                                      l_param :LPARAM) -> LRESULT
{
    if msg == winapi::winuser::WM_MENUCOMMAND {
        WININFO_STASH.with(|stash| {
            let stash = stash.borrow();
            let stash = stash.as_ref();
            if let Some(stash) = stash {
                let menuId = user32::GetMenuItemID(stash.info.hmenu,
                                                   w_param as i32) as i32;
                if menuId != -1 {
                    stash.tx.send(SystrayEvent {
                        menu_index: menuId as u32,
                        menu_checked: false
                    });
                }
            }
        });
    }

    if msg == winapi::winuser::WM_USER + 1 {
        if l_param as UINT == winapi::winuser::WM_LBUTTONUP  {
            let mut p = winapi::POINT {
                x: 0,
                y: 0
            };
            if user32::GetCursorPos(&mut p as *mut winapi::POINT) == 0 {
                return 1;
            }
            user32::SetForegroundWindow(h_wnd);
            WININFO_STASH.with(|stash| {
                let stash = stash.borrow();
                let stash = stash.as_ref();
                if let Some(stash) = stash {
                    user32::TrackPopupMenu(stash.info.hmenu,
                                           0,
                                           p.x,
                                           p.y,
                                           (winapi::TPM_BOTTOMALIGN | winapi::TPM_LEFTALIGN) as i32,
                                           h_wnd,
                                           std::ptr::null_mut());
                }
            });
        }
    }
    if msg == winapi::winuser::WM_DESTROY {
        user32::PostQuitMessage(0);
    }
    return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
}
// Would be nice to have default for the notify icon struct, since there's a lot
// of setup code otherwise. To get around orphan trait error, define trait here.
trait Default {
    fn default() -> Self;
}

impl Default for winapi::shellapi::NOTIFYICONDATAA {
    fn default() -> winapi::shellapi::NOTIFYICONDATAA {
        unsafe {
            winapi::shellapi::NOTIFYICONDATAA {
                cbSize: std::mem::size_of::<winapi::shellapi::NOTIFYICONDATAA>() as DWORD,
                hWnd: 0 as HWND,
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
}

unsafe fn init_window() -> Result<WindowInfo, SystrayError> {
    let class_name = to_wstring("my_window");
    let hinstance : HINSTANCE = kernel32::GetModuleHandleA(std::ptr::null_mut());
    let wnd = WNDCLASSW {
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
    let hwnd = user32::CreateWindowExW(0,
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
    let mut nid = winapi::shellapi::NOTIFYICONDATAA::default();
    nid.hWnd = hwnd;
    nid.uID = 0x1;
    nid.uFlags = winapi::NIF_MESSAGE;
    nid.uCallbackMessage = winapi::WM_USER + 1;
    println!("Adding icon! {}", shell32::Shell_NotifyIconA(winapi::NIM_ADD,
                                                           &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));
    // Setup menu
    let hmenu = user32::CreatePopupMenu();
    let m = winapi::MENUINFO {
        cbSize: std::mem::size_of::<winapi::MENUINFO>() as DWORD,
        fMask: winapi::MIM_APPLYTOSUBMENUS | winapi::MIM_STYLE,
        dwStyle: winapi::MNS_NOTIFYBYPOS,
        cyMax: 0 as UINT,
        hbrBack: 0 as HBRUSH,
        dwContextHelpID: 0 as DWORD,
        dwMenuData: 0 as winapi::ULONG_PTR
    };
    println!("Created menu! {}", user32::SetMenuInfo(hmenu, &m as *const winapi::MENUINFO));

    Ok(WindowInfo {
        hwnd: hwnd,
        hmenu: hmenu,
        hinstance: hinstance,
    })
}

unsafe fn run_loop() {
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
        println!("RUNNING LOOP");
        user32::GetMessageW(&mut msg, 0 as HWND, 0, 0);
        if msg.message == winapi::winuser::WM_QUIT {
            println!("QUITTING LOOP");
            break;
        }
        println!("{}", msg.message);
        user32::TranslateMessage(&mut msg);
        user32::DispatchMessageW(&mut msg);
    }
}

pub struct Window {
    info: WindowInfo,
    windows_loop: Option<thread::JoinHandle<()>>,
    menu_idx: u32,
    pub callback: HashMap<u32, Callback>,
    pub rx: Receiver<SystrayEvent>,
}

impl Window {
    pub fn new() -> Result<Window, SystrayError> {
        let (tx, rx) = channel();
        let (event_tx, event_rx) = channel();
        let windows_loop = thread::spawn(move || {
            unsafe {
                let i = init_window();
                let k;
                match i {
                    Ok(j) => {
                        tx.send(Ok(j.clone())).ok();
                        k = j;
                    }
                    Err(e) => {
                        // If creation didn't work, return out of the thread.
                        tx.send(Err(e)).ok();
                        return;
                    }
                };
                WININFO_STASH.with(|stash| {
                    let data = WindowsLoopData {
                        info: k,
                        tx: event_tx
                    };
                    (*stash.borrow_mut()) = Some(data);
                });
                run_loop();
            }
        });
        let info = match rx.recv().unwrap() {
            Ok(i) => i,
            Err(e) => {
                panic!(e);
            }
        };
        let mut w = Window {
            info: info,
            windows_loop: Some(windows_loop),
            rx: event_rx,
            menu_idx: 0,
            callback: HashMap::new()
        };
        // TODO This really shouldn't be compulsory. Need to figure out how to
        // make closures that can wrap around window object and know it.
        w.add_menu_entry(&"Quit".to_string());
        w.add_menu_separator();
        Ok(w)
    }

    pub fn quit(&mut self) {
        unsafe {
            user32::PostMessageW(self.info.hwnd, winapi::WM_DESTROY,
                                 0 as WPARAM, 0 as LPARAM);
        }
        if let Some(t) = self.windows_loop.take() {
            t.join().ok();
        }
    }

    pub fn set_tooltip(&self, tooltip: &String) {
        // Add Tooltip
        // Gross way to convert String to [i8; 128]
        // TODO: Clean up conversion, test for length so we don't panic at runtime
        let tt = tooltip.as_bytes().clone();
        let mut nid = winapi::shellapi::NOTIFYICONDATAA::default();
        nid.hWnd = self.info.hwnd;
        for i in 0..tt.len() {
            nid.szTip[i] = tt[i] as i8;
        }
        nid.uFlags = winapi::NIF_TIP;
        unsafe {
            println!("Setting tip! {}", shell32::Shell_NotifyIconA(winapi::NIM_MODIFY,
                                                                   &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));
        }
    }

    fn add_menu_entry(&mut self, item_name: &String) {
        let mut st = to_wstring(item_name);
        let idx = self.menu_idx;
        self.menu_idx += 1;
        let item = winapi::MENUITEMINFOW {
            cbSize: std::mem::size_of::<winapi::MENUITEMINFOW>() as UINT,
            fMask: (winapi::MIIM_FTYPE | winapi::MIIM_STRING | winapi::MIIM_ID | winapi::MIIM_STATE),
            fType: winapi::MFT_STRING,
            fState: 0 as UINT,
            wID: idx as UINT,
            hSubMenu: 0 as HMENU,
            hbmpChecked: 0 as HBITMAP,
            hbmpUnchecked: 0 as HBITMAP,
            dwItemData: 0 as u64,
            dwTypeData: st.as_mut_ptr(),
            cch: (item_name.len() * 2) as u32, // 16 bit characters
            hbmpItem: 0 as HBITMAP
        };
        unsafe {
            user32::InsertMenuItemW(self.info.hmenu, 0, 1, &item as *const winapi::MENUITEMINFOW);
        }
    }

    pub fn add_menu_separator(&mut self) {
        let idx = self.menu_idx;
        self.menu_idx += 1;
        let item = winapi::MENUITEMINFOW {
            cbSize: std::mem::size_of::<winapi::MENUITEMINFOW>() as UINT,
            fMask: winapi::MIIM_FTYPE,
            fType: winapi::MFT_SEPARATOR,
            fState: 0 as UINT,
            wID: idx as UINT,
            hSubMenu: 0 as HMENU,
            hbmpChecked: 0 as HBITMAP,
            hbmpUnchecked: 0 as HBITMAP,
            dwItemData: 0 as u64,
            dwTypeData: std::ptr::null_mut(),
            cch: 0 as u32, // 16 bit characters
            hbmpItem: 0 as HBITMAP
        };
        unsafe {
            user32::InsertMenuItemW(self.info.hmenu, 0, 1, &item as *const winapi::MENUITEMINFOW);
        }
    }

    pub fn add_menu_item<F>(&mut self, item_name: &String, f: F)
        where F: std::ops::Fn<(),Output=()> + 'static {
        let mut st = to_wstring(item_name);
        let idx = self.menu_idx;
        self.menu_idx += 1;
        let item = winapi::MENUITEMINFOW {
            cbSize: std::mem::size_of::<winapi::MENUITEMINFOW>() as UINT,
            fMask: (winapi::MIIM_FTYPE | winapi::MIIM_STRING | winapi::MIIM_ID | winapi::MIIM_STATE),
            fType: winapi::MFT_STRING,
            fState: 0 as UINT,
            wID: idx as UINT,
            hSubMenu: 0 as HMENU,
            hbmpChecked: 0 as HBITMAP,
            hbmpUnchecked: 0 as HBITMAP,
            dwItemData: 0 as u64,
            dwTypeData: st.as_mut_ptr(),
            cch: (item_name.len() * 2) as u32, // 16 bit characters
            hbmpItem: 0 as HBITMAP
        };
        self.callback.insert(idx, make_callback(f));
        unsafe {
            user32::InsertMenuItemW(self.info.hmenu, 0, 1, &item as *const winapi::MENUITEMINFOW);
        }
    }

    fn set_icon(&self, icon: HICON) {
        unsafe {
            let mut nid = winapi::shellapi::NOTIFYICONDATAA::default();
            nid.hWnd = self.info.hwnd;
            nid.uFlags = winapi::NIF_ICON;
            nid.hIcon = icon;
            println!("Setting icon! {}", shell32::Shell_NotifyIconA(winapi::NIM_MODIFY,
                                                                    &mut nid as *mut winapi::shellapi::NOTIFYICONDATAA));
        }
    }

    pub fn wait_for_message(&mut self) {
        loop {
            let msg = self.rx.recv().unwrap();
            println!("Got {}", msg.menu_index);
            if msg.menu_index == 0 {
                self.quit();
                return;
            }
            if self.callback.contains_key(&msg.menu_index) {
                self.callback.get(&msg.menu_index).map(|fun| fun());
            }
        }
    }

    pub fn set_icon_from_resource(&self, resource_name: &String) {
        let icon;
        unsafe {
            icon = user32::LoadImageW(self.info.hinstance,
                                      to_wstring(&resource_name).as_ptr(),
                                      winapi::IMAGE_ICON,
                                      64,
                                      64,
                                      0) as HICON;
        }
        self.set_icon(icon);
    }

    pub fn set_icon_from_file(&self, icon_file: &String) {
        let wstr_icon_file = to_wstring(&icon_file);
        let hicon;
        unsafe {
            hicon = user32::LoadImageW(std::ptr::null_mut() as HINSTANCE, wstr_icon_file.as_ptr(),
                                       winapi::IMAGE_ICON, 64, 64, winapi::LR_LOADFROMFILE) as HICON;
        }
        if hicon == std::ptr::null_mut() as HICON {
            // TODO Throw Error
            return;
        }
        self.set_icon(hicon);
    }
}
