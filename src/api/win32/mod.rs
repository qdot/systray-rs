use {SystrayEvent, SystrayError, Callback, make_callback};
use std;
use std::cell::{Cell, RefCell};
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
use winapi::minwindef::{UINT, DWORD, WPARAM, LPARAM, LRESULT, HINSTANCE, TRUE, PBYTE};
use winapi::winuser::{WNDCLASSW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, LR_DEFAULTCOLOR};

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

unsafe fn get_win_os_error(msg: &str) -> SystrayError {
    SystrayError::OsError(format!("{}: {}", &msg, kernel32::GetLastError()))
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
                let menu_id = user32::GetMenuItemID(stash.info.hmenu,
                                                    w_param as i32) as i32;
                if menu_id != -1 {
                    stash.tx.send(SystrayEvent {
                        menu_index: menu_id as u32,
                        menu_checked: false
                    }).ok();
                }
            }
        });
    }

    if msg == winapi::winuser::WM_USER + 1 {
        if l_param as UINT == winapi::winuser::WM_LBUTTONUP ||
            l_param as UINT == winapi::winuser::WM_RBUTTONUP {
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

fn get_nid_struct(hwnd : &HWND) -> winapi::shellapi::NOTIFYICONDATAW {
    winapi::shellapi::NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<winapi::shellapi::NOTIFYICONDATAW>() as DWORD,
        hWnd: *hwnd,
        uID: 0x1 as UINT,
        uFlags: 0 as UINT,
        uCallbackMessage: 0 as UINT,
        hIcon: 0 as HICON,
        szTip: [0 as u16; 128],
        dwState: 0 as DWORD,
        dwStateMask: 0 as DWORD,
        szInfo: [0 as u16; 256],
        uTimeout: 0 as UINT,
        szInfoTitle: [0 as u16; 64],
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

fn get_menu_item_struct() -> winapi::MENUITEMINFOW {
    winapi::MENUITEMINFOW {
        cbSize: std::mem::size_of::<winapi::MENUITEMINFOW>() as UINT,
        fMask: 0 as UINT,
        fType: 0 as UINT,
        fState: 0 as UINT,
        wID: 0 as UINT,
        hSubMenu: 0 as HMENU,
        hbmpChecked: 0 as HBITMAP,
        hbmpUnchecked: 0 as HBITMAP,
        dwItemData: 0 as winapi::ULONG_PTR,
        dwTypeData: std::ptr::null_mut(),
        cch: 0 as u32,
        hbmpItem: 0 as HBITMAP
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
    if user32::RegisterClassW(&wnd) == 0 {
        return Err(get_win_os_error("Error creating window class"));
    }
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
    if hwnd == std::ptr::null_mut() {
        return Err(get_win_os_error("Error creating window"));
    }
    let mut nid = get_nid_struct(&hwnd);
    nid.uID = 0x1;
    nid.uFlags = winapi::NIF_MESSAGE;
    nid.uCallbackMessage = winapi::WM_USER + 1;
    if shell32::Shell_NotifyIconW(winapi::NIM_ADD,
                                  &mut nid as *mut winapi::shellapi::NOTIFYICONDATAW) == 0 {
        return Err(get_win_os_error("Error adding menu icon"));
    }
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
    if user32::SetMenuInfo(hmenu, &m as *const winapi::MENUINFO) == 0 {
        return Err(get_win_os_error("Error setting up menu"));
    }

    Ok(WindowInfo {
        hwnd: hwnd,
        hmenu: hmenu,
        hinstance: hinstance,
    })
}

unsafe fn run_loop() {
    debug!("Running windows loop");
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
        user32::GetMessageW(&mut msg, 0 as HWND, 0, 0);
        if msg.message == winapi::winuser::WM_QUIT {
            break;
        }
        user32::TranslateMessage(&mut msg);
        user32::DispatchMessageW(&mut msg);
    }
    debug!("Leaving windows run loop");
}

pub struct Window {
    info: WindowInfo,
    windows_loop: Option<thread::JoinHandle<()>>,
    menu_idx: Cell<u32>,
    callback: RefCell<HashMap<u32, Callback>>,
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
                return Err(e);
            }
        };
        let w = Window {
            info: info,
            windows_loop: Some(windows_loop),
            rx: event_rx,
            menu_idx: Cell::new(0),
            callback: RefCell::new(HashMap::new())
        };
        Ok(w)
    }

    pub fn quit(&self) {
        unsafe {
            user32::PostMessageW(self.info.hwnd, winapi::WM_DESTROY,
                                 0 as WPARAM, 0 as LPARAM);
        }
    }


    pub fn set_tooltip(&self, tooltip: &String) -> Result<(), SystrayError> {
        // Add Tooltip
        debug!("Setting tooltip to {}", tooltip);
        // Gross way to convert String to [i8; 128]
        // TODO: Clean up conversion, test for length so we don't panic at runtime
        let tt = tooltip.as_bytes().clone();
        let mut nid = get_nid_struct(&self.info.hwnd);
        for i in 0..tt.len() {
            nid.szTip[i] = tt[i] as u16;
        }
        nid.uFlags = winapi::NIF_TIP;
        unsafe {
            if shell32::Shell_NotifyIconW(winapi::NIM_MODIFY,
                                          &mut nid as *mut winapi::shellapi::NOTIFYICONDATAW) == 0 {
                return Err(get_win_os_error("Error setting tooltip"));
            }
        }
        Ok(())
    }

    fn add_menu_entry(&self, item_name: &String) -> Result<u32, SystrayError> {
        let mut st = to_wstring(item_name);
        let idx = self.menu_idx.get();
        self.menu_idx.set(idx + 1);
        let mut item = get_menu_item_struct();
        item.fMask = winapi::MIIM_FTYPE | winapi::MIIM_STRING | winapi::MIIM_ID | winapi::MIIM_STATE;
        item.fType = winapi::MFT_STRING;
        item.wID = idx;
        item.dwTypeData = st.as_mut_ptr();
        item.cch = (item_name.len() * 2) as u32;
        unsafe {
            if user32::InsertMenuItemW(self.info.hmenu,
                                       idx,
                                       1,
                                       &item as *const winapi::MENUITEMINFOW) == 0 {
                return Err(get_win_os_error("Error inserting menu item"));
            }
        }
        Ok(idx)
    }

    pub fn add_menu_separator(&self) -> Result<u32, SystrayError> {
        let idx = self.menu_idx.get();
        self.menu_idx.set(idx + 1);
        let mut item = get_menu_item_struct();
        item.fMask = winapi::MIIM_FTYPE;
        item.fType = winapi::MFT_SEPARATOR;
        item.wID = idx;
        unsafe {
            if user32::InsertMenuItemW(self.info.hmenu,
                                       idx,
                                       1,
                                       &item as *const winapi::MENUITEMINFOW) == 0 {
                return Err(get_win_os_error("Error inserting separator"));
            }
        }
        Ok(idx)
    }

    pub fn add_menu_item<F>(&self, item_name: &String, f: F) -> Result<u32, SystrayError>
        where F: std::ops::Fn(&Window) -> () + 'static {
        let idx = match self.add_menu_entry(item_name) {
            Ok(i) => i,
            Err(e) => {
                return Err(e);
            }
        };
        let mut m = self.callback.borrow_mut();
        m.insert(idx, make_callback(f));
        Ok(idx)
    }

    fn set_icon(&self, icon: HICON) -> Result<(), SystrayError> {
        unsafe {
            let mut nid = get_nid_struct(&self.info.hwnd);
            nid.uFlags = winapi::NIF_ICON;
            nid.hIcon = icon;
            if shell32::Shell_NotifyIconW(winapi::NIM_MODIFY,
                                          &mut nid as *mut winapi::shellapi::NOTIFYICONDATAW) == 0 {
                return Err(get_win_os_error("Error setting icon"));
            }
        }
        Ok(())
    }

    pub fn wait_for_message(&mut self) {
        loop {
            let msg;
            match self.rx.recv() {
                Ok(m) => msg = m,
                Err(_) => {
                    // If self.rx fails, we're in thread shutdown. Join here.
                    if let Some(t) = self.windows_loop.take() {
                        t.join().ok();
                    }
                    break;
                }
            }
            if (*self.callback.borrow()).contains_key(&msg.menu_index) {
                let f = (*self.callback.borrow_mut()).remove(&msg.menu_index).unwrap();
                f(&self);
                (*self.callback.borrow_mut()).insert(msg.menu_index, f);
            }
        }
    }

    pub fn set_icon_from_resource(&self, resource_name: &String) -> Result<(), SystrayError> {
        let icon;
        unsafe {
            icon = user32::LoadImageW(self.info.hinstance,
                                      to_wstring(&resource_name).as_ptr(),
                                      winapi::IMAGE_ICON,
                                      64,
                                      64,
                                      0) as HICON;
            if icon == std::ptr::null_mut() as HICON {
                return Err(get_win_os_error("Error setting icon from resource"));
            }
        }
        self.set_icon(icon)
    }

    pub fn set_icon_from_file(&self, icon_file: &String) -> Result<(), SystrayError> {
        let wstr_icon_file = to_wstring(&icon_file);
        let hicon;
        unsafe {
            hicon = user32::LoadImageW(std::ptr::null_mut() as HINSTANCE, wstr_icon_file.as_ptr(),
                                       winapi::IMAGE_ICON, 64, 64, winapi::LR_LOADFROMFILE) as HICON;
            if hicon == std::ptr::null_mut() as HICON {
                return Err(get_win_os_error("Error setting icon from file"));
            }
        }
        self.set_icon(hicon)
    }

    pub fn set_icon_from_buffer(&self, buffer: &[u8], width: u32, height: u32) -> Result<(), SystrayError> {
        let offset = unsafe {
            user32::LookupIconIdFromDirectoryEx(
                buffer.as_ptr() as PBYTE,
                TRUE,
                width as i32,
                height as i32,
                LR_DEFAULTCOLOR
            )
        };

        if offset != 0 {
            let icon_data = &buffer[offset as usize ..];
            let hicon = unsafe {
                user32::CreateIconFromResourceEx(
                    icon_data.as_ptr() as PBYTE,
                    0,
                    TRUE,
                    0x30000,
                    width as i32,
                    height as i32,
                    LR_DEFAULTCOLOR
                )
            };

            if hicon == std::ptr::null_mut() as HICON {
                return Err( unsafe { get_win_os_error("Cannot load icon from the buffer") } );
            }

            self.set_icon(hicon)
        } else {
            Err( unsafe { get_win_os_error("Error setting icon from buffer") })
        }
    }

    pub fn shutdown(&self) -> Result<(), SystrayError> {
        unsafe {
            let mut nid = get_nid_struct(&self.info.hwnd);
            nid.uFlags = winapi::NIF_ICON;
            if shell32::Shell_NotifyIconW(winapi::NIM_DELETE,
                                          &mut nid as *mut winapi::shellapi::NOTIFYICONDATAW) == 0 {
                return Err(get_win_os_error("Error deleting icon from menu"));
            }
        }
        Ok(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}
