// Until winapi hits 0.3 on crates.io, add these so we can publish a crate.
#![allow(dead_code)]
#![allow(non_snake_case)]

use winapi::{DWORD, LPMENUITEMINFOA, LPMENUITEMINFOW, c_int, RECT, UINT, BOOL, ULONG_PTR, CHAR, GUID, WCHAR};
use winapi::windef::{HWND, HMENU, HICON, HBRUSH, HBITMAP};

macro_rules! UNION {
    ($base:ident, $field:ident, $variant:ident, $variantmut:ident, $fieldtype:ty) => {
        impl $base {
            #[inline]
            pub unsafe fn $variant(&self) -> &$fieldtype {
                ::std::mem::transmute(&self.$field)
            }
            #[inline]
            pub unsafe fn $variantmut(&mut self) -> &mut $fieldtype {
                ::std::mem::transmute(&mut self.$field)
            }
        }
    }
}

macro_rules! STRUCT {
    {$(#[$attrs:meta])* nodebug struct $name:ident { $($field:ident: $ftype:ty,)+ }} => {
        #[repr(C)] $(#[$attrs])*
        pub struct $name {
            $(pub $field: $ftype,)+
        }
        impl Copy for $name {}
        impl Clone for $name { fn clone(&self) -> $name { *self } }
    };
    {$(#[$attrs:meta])* struct $name:ident { $($field:ident: $ftype:ty,)+ }} => {
        #[repr(C)] #[derive(Debug)] $(#[$attrs])*
        pub struct $name {
            $(pub $field: $ftype,)+
        }
        impl Copy for $name {}
        impl Clone for $name { fn clone(&self) -> $name { *self } }
    };
}

extern "system" {
    pub fn GetMenuInfo(hMenu: HMENU, lpcmi: LPMENUINFO) -> BOOL;
    pub fn GetMenuItemCount(hMenu: HMENU) -> c_int;
    pub fn GetMenuItemID(hMenu: HMENU, nPos: c_int) -> UINT;
    pub fn GetMenuItemInfoA(hMenu: HMENU, uItem: UINT, fByPosition: BOOL, lpmii: LPMENUITEMINFOA) -> BOOL;
    pub fn GetMenuItemInfoW(hMenu: HMENU, uItem: UINT, fByPosition: BOOL, lpmii: LPMENUITEMINFOW) -> BOOL;
    pub fn SetMenuInfo(hMenu: HMENU, lpcmi: LPCMENUINFO) -> BOOL;
    pub fn TrackPopupMenu(hMenu: HMENU, uFlags: UINT, x: c_int, y: c_int, nReserved: c_int,
                          hWnd: HWND, prcRect: *const RECT);
    pub fn TrackPopupMenuEx(hMenu: HMENU, fuFlags: UINT, x: c_int, y: c_int, hWnd: HWND,
                            lptpm: LPTPMPARAMS);
    pub fn Shell_NotifyIconA(dwMessage: DWORD, lpData: PNOTIFYICONDATAA) -> BOOL;
    pub fn Shell_NotifyIconW(dwMessage: DWORD, lpData: PNOTIFYICONDATAW) -> BOOL;
}


pub const NIM_ADD: DWORD = 0x00000000;
pub const NIM_MODIFY: DWORD = 0x00000001;
pub const NIM_DELETE: DWORD = 0x00000002;
pub const NIM_SETFOCUS: DWORD = 0x00000003;
pub const NIM_SETVERSION: DWORD = 0x00000004;
pub const NIF_MESSAGE: UINT = 0x00000001;
pub const NIF_ICON: UINT = 0x00000002;
pub const NIF_TIP: UINT = 0x00000004;
pub const NIF_STATE: UINT = 0x00000008;
pub const NIF_INFO: UINT = 0x00000010;
pub const NIF_GUID: UINT = 0x00000020;
pub const NIF_REALTIME: UINT = 0x00000040;
pub const NIF_SHOWTIP: UINT = 0x00000080;
pub const NOTIFYICON_VERSION: UINT = 3;
pub const NOTIFYICON_VERSION_4: UINT = 4;

STRUCT!{nodebug struct NOTIFYICONDATAA {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [CHAR; 128],
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [CHAR; 256],
    uTimeout: UINT,
    szInfoTitle: [CHAR; 64],
    dwInfoFlags: DWORD,
    guidItem: GUID,
    hBalloonIcon: HICON,
}}
UNION!(NOTIFYICONDATAA, uTimeout, uTimeout, uTimeout_mut, UINT);
UNION!(NOTIFYICONDATAA, uTimeout, uVersion, uVersion_mut, UINT);
pub type PNOTIFYICONDATAA = *mut NOTIFYICONDATAA;

STRUCT!{nodebug struct NOTIFYICONDATAW {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [WCHAR; 128],
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [WCHAR; 256],
    uTimeout: UINT,
    szInfoTitle: [WCHAR; 64],
    dwInfoFlags: DWORD,
    guidItem: GUID,
    hBalloonIcon: HICON,
}}
UNION!(NOTIFYICONDATAW, uTimeout, uTimeout, uTimeout_mut, UINT);
UNION!(NOTIFYICONDATAW, uTimeout, uVersion, uVersion_mut, UINT); // used with NIM_SETVERSION, values 0, 3 and 4

pub type PNOTIFYICONDATAW = *mut NOTIFYICONDATAW;
pub const MIIM_BITMAP: UINT = 0x00000080;
pub const MIIM_CHECKMARKS: UINT = 0x00000008;
pub const MIIM_DATA: UINT = 0x00000020;
pub const MIIM_FTYPE: UINT = 0x00000100;
pub const MIIM_ID: UINT = 0x00000002;
pub const MIIM_STATE: UINT = 0x00000001;
pub const MIIM_STRING: UINT = 0x00000040;
pub const MIIM_SUBMENU: UINT = 0x00000004;
pub const MIIM_TYPE: UINT = 0x00000010;

pub const MFT_BITMAP: UINT = 0x00000004;
pub const MFT_MENUBARBREAK: UINT = 0x00000020;
pub const MFT_MENUBREAK: UINT = 0x00000040;
pub const MFT_OWNERDRAW: UINT = 0x00000100;
pub const MFT_RADIOCHECK: UINT = 0x00000200;
pub const MFT_RIGHTJUSTIFY: UINT = 0x00004000;
pub const MFT_RIGHTORDER: UINT = 0x00002000;
pub const MFT_SEPARATOR: UINT = 0x00000800;
pub const MFT_STRING: UINT = 0x00000000;

pub const MFS_CHECKED: UINT = 0x00000008;
pub const MFS_DEFAULT: UINT = 0x00001000;
pub const MFS_DISABLED: UINT = 0x00000003;
pub const MFS_ENABLED: UINT = 0x00000000;
pub const MFS_GRAYED: UINT = 0x00000003;
pub const MFS_HILITE: UINT = 0x00000080;
pub const MFS_UNCHECKED: UINT = 0x00000000;
pub const MFS_UNHILITE: UINT = 0x00000000;

//pub const HBMMENU_CALLBACK: HBITMAP = -1 as HBITMAP;
pub const HBMMENU_MBAR_CLOSE: HBITMAP = 5 as HBITMAP;
pub const HBMMENU_MBAR_CLOSE_D: HBITMAP = 6 as HBITMAP;
pub const HBMMENU_MBAR_MINIMIZE: HBITMAP = 3 as HBITMAP;
pub const HBMMENU_MBAR_MINIMIZE_D: HBITMAP = 7 as HBITMAP;
pub const HBMMENU_MBAR_RESTORE: HBITMAP = 2 as HBITMAP;
pub const HBMMENU_POPUP_CLOSE: HBITMAP = 8 as HBITMAP;
pub const HBMMENU_POPUP_MAXIMIZE: HBITMAP = 10 as HBITMAP;
pub const HBMMENU_POPUP_MINIMIZE: HBITMAP = 11 as HBITMAP;
pub const HBMMENU_POPUP_RESTORE: HBITMAP = 9 as HBITMAP;
pub const HBMMENU_SYSTEM: HBITMAP = 1 as HBITMAP;

pub const MIM_MAXHEIGHT: UINT = 0x00000001;
pub const MIM_BACKGROUND: UINT = 0x00000002;
pub const MIM_HELPID: UINT = 0x00000004;
pub const MIM_MENUDATA: UINT = 0x00000008;
pub const MIM_STYLE: UINT = 0x00000010;
pub const MIM_APPLYTOSUBMENUS: UINT = 0x80000000;

pub const MNS_CHECKORBMP: UINT = 0x04000000;
pub const MNS_NOTIFYBYPOS: UINT = 0x08000000;
pub const MNS_AUTODISMISS: UINT = 0x10000000;
pub const MNS_DRAGDROP: UINT = 0x20000000;
pub const MNS_MODELESS: UINT = 0x40000000;
pub const MNS_NOCHECK: UINT = 0x80000000;

STRUCT!{struct MENUINFO {
    cbSize: DWORD,
    fMask: DWORD,
    dwStyle: DWORD,
    cyMax: UINT,
    hbrBack: HBRUSH,
    dwContextHelpID: DWORD,
    dwMenuData: ULONG_PTR,
}}
pub type LPMENUINFO = *mut MENUINFO;
pub type LPCMENUINFO = *const MENUINFO;

pub const TPM_LEFTALIGN: UINT = 0x0000;
pub const TPM_CENTERALIGN: UINT = 0x0004;
pub const TPM_RIGHTALIGN: UINT = 0x0008;
pub const TPM_TOPALIGN: UINT = 0x0000;
pub const TPM_VCENTERALIGN: UINT = 0x0010;
pub const TPM_BOTTOMALIGN: UINT = 0x0020;
pub const TPM_NONOTIFY: UINT = 0x0080;
pub const TPM_RETURNCMD: UINT = 0x0100;
pub const TPM_LEFTBUTTON: UINT = 0x0000;
pub const TPM_RIGHTBUTTON: UINT = 0x0002;
pub const TPM_HORNEGANIMATION: UINT = 0x0800;
pub const TPM_HORPOSANIMATION: UINT = 0x0400;
pub const TPM_NOANIMATION: UINT = 0x4000;
pub const TPM_VERNEGANIMATION: UINT = 0x2000;
pub const TPM_VERPOSANIMATION: UINT = 0x1000;

STRUCT!{struct TPMPARAMS {
    cbSize: UINT,
    rcExclude: RECT,
}}

pub type LPTPMPARAMS = *const TPMPARAMS;
