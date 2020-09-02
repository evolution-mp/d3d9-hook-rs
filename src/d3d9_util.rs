use {
  crate::process,
  std::{mem, ptr},
  winapi::{
    shared::{
      d3d9::{Direct3DCreate9, D3D_SDK_VERSION, IDirect3DDevice9, D3DADAPTER_DEFAULT, D3DCREATE_SOFTWARE_VERTEXPROCESSING},
      d3d9types::{D3DPRESENT_PARAMETERS, D3DSWAPEFFECT_DISCARD, D3DDEVTYPE_HAL},
      minwindef::FALSE,
    },
  },
};

const DIRECTX_VTABLE_SIZE: usize = 119;

pub unsafe fn get_d3d9_vtable() -> Result<Vec<*const usize>, &'static str> {
  let p_d3d = Direct3DCreate9(D3D_SDK_VERSION);
  if p_d3d.is_null() {
    return Err("Direct3DCreate9 returned null");
  }

  let proc_hwnd = match process::get_process_window() {
    Some(hwnd) => hwnd,
    _ => return Err("Failed to find hwnd"),
  };

  let p_dummy_device: *mut IDirect3DDevice9 = ptr::null_mut();
  let mut d3dpp = D3DPRESENT_PARAMETERS {
    BackBufferWidth: 0,
    BackBufferHeight: 0,
    BackBufferFormat: 0,
    BackBufferCount: 0,
    MultiSampleType: 0,
    MultiSampleQuality: 0,
    SwapEffect: D3DSWAPEFFECT_DISCARD,
    hDeviceWindow: proc_hwnd,
    Windowed: FALSE,
    EnableAutoDepthStencil: 0,
    AutoDepthStencilFormat: 0,
    Flags: 0,
    FullScreen_RefreshRateInHz: 0,
    PresentationInterval: 0
  };

  let mut dummy_device_created = (*p_d3d).CreateDevice(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL,
                                                       d3dpp.hDeviceWindow, D3DCREATE_SOFTWARE_VERTEXPROCESSING,
                                                       mem::transmute(&d3dpp),
                                                       mem::transmute(&p_dummy_device));

  if dummy_device_created != 0 {
    d3dpp.Windowed = !d3dpp.Windowed;
    dummy_device_created = (*p_d3d).CreateDevice(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL,
                                                 d3dpp.hDeviceWindow, D3DCREATE_SOFTWARE_VERTEXPROCESSING,
                                                 mem::transmute(&d3dpp),
                                                 mem::transmute(&p_dummy_device));
    if dummy_device_created != 0 {
      return Err("Failed to create dummy_device");
    }
  }

  let v = std::slice::from_raw_parts((p_dummy_device as *const *const *const usize).read(), DIRECTX_VTABLE_SIZE).to_vec();
  return match v.is_empty() {
    true => Err("Failed to dump d3d9 device addresses"),
    false => Ok(v),
  }
}