use {
  std::thread,
  winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE},
    um::{
      consoleapi::AllocConsole,
      libloaderapi::DisableThreadLibraryCalls,
      winnt::DLL_PROCESS_ATTACH,
    }
  }
};

mod d3d9_util;
mod hook;
mod process;

unsafe fn init() {
  AllocConsole();
  let hwnd = match process::get_process_window() {
    Some(hwnd) => hwnd,
    _ => panic!("Failed to find current process windo hwnd"),
  };
  let result = d3d9_util::get_d3d9_vtable(hwnd);
  match result {
    Ok(v) => {
      println!("d3d9Device[42]: {:p}", *v.get(42).unwrap());
      hook::hook_device_functions(v);
      hook::hook_wnd_proc(hwnd);
    },
    Err(s) => println!("Error finding vtable addresses: {}", s),
  }
}

#[no_mangle]
pub extern "stdcall" fn DllMain(h_inst: HINSTANCE, fdw_reason: DWORD, _: LPVOID) -> BOOL {
  if fdw_reason == DLL_PROCESS_ATTACH {
    unsafe {
      DisableThreadLibraryCalls(h_inst);
    };
    thread::spawn(|| unsafe { init() });
  }
  return TRUE
}