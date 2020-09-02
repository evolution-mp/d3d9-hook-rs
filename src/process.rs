use {
  std::ptr,
  winapi::{
    shared::{
      minwindef::{BOOL, DWORD, FALSE, LPARAM, TRUE},
      windef::HWND,
    },
    um::{
      processthreadsapi::GetCurrentProcessId,
      winuser::{EnumWindows, GetWindowThreadProcessId},
    }
  },
};

pub unsafe fn get_process_window() -> Option<HWND> {
  extern "system" fn enum_windows_callback(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let mut wnd_proc_id: DWORD = 0;
    unsafe {
      GetWindowThreadProcessId(hwnd, &mut wnd_proc_id as *mut DWORD);
      if GetCurrentProcessId() != wnd_proc_id {
        return TRUE;
      }
      *(l_param as *mut HWND) = hwnd;
    }
    return FALSE;
  }

  let mut hwnd: HWND = ptr::null_mut();
  EnumWindows(Some(enum_windows_callback), &mut hwnd as *mut HWND as LPARAM);
  return if hwnd.is_null() {
    None
  } else {
    Some(hwnd)
  }
}