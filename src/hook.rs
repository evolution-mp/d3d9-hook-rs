use std::ptr::NonNull;

use imgui::*;
use imgui_dx9_renderer::{Renderer, RendererError};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM, FALSE, BOOL, TRUE};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONUP, WM_MBUTTONUP, WM_MOUSEWHEEL, WM_MOUSEMOVE, WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP, WM_CHAR, WNDPROC, CallWindowProcW, GWLP_WNDPROC, SetWindowLongPtrW, GET_WHEEL_DELTA_WPARAM};

use {
  detour::{Error, GenericDetour},
  std::mem,
  winapi::{
    shared::{
      d3d9::LPDIRECT3DDEVICE9,
      d3d9types::{D3DCLEAR_TARGET, D3DCOLOR_XRGB, D3DRECT},
    },
    um::winnt::HRESULT,
  },
};

use crate::d3d9_util::HWND_RECT;
use winapi::shared::basetsd::LONG_PTR;
use std::ops::Shr;

static mut ENDSCENE_DETOUR: Result<GenericDetour<EndScene>, Error> = Err(Error::NotInitialized);
static mut IMGUI_CONTEXT: Option<Context> = None;
static mut IMGUI_RENDERER: Result<Renderer, RendererError> = Err(RendererError::OutOfMemory);
static mut O_WND_PROC: Option<WNDPROC> = None;
static mut INIT: bool = false;

// Note: this isn't a complete port of imgui's wndproc implementation. This works and I don't care enough to fix it.
// A better alternative would be to patch winit to support custom hwnd
// This doesn't handle dragging
pub unsafe extern "stdcall" fn hk_wnd_proc(hwnd: HWND, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
  if O_WND_PROC.is_none() || IMGUI_CONTEXT.is_none() {
    return FALSE as LRESULT
  }

  let mut io = IMGUI_CONTEXT.as_mut().unwrap().io_mut();
  let ret: BOOL = match u_msg {
    WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
      match u_msg {
        WM_LBUTTONDOWN => io.mouse_down[0] = true,
        WM_RBUTTONDOWN => io.mouse_down[1] = true,
        WM_MBUTTONDOWN => io.mouse_down[2] = true,
        _ => ()
      }
      TRUE
    },
    WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
      match u_msg {
        WM_LBUTTONDOWN => io.mouse_down[0] = false,
        WM_RBUTTONDOWN => io.mouse_down[1] = false,
        WM_MBUTTONDOWN => io.mouse_down[2] = false,
        _ => ()
      }
      TRUE
    },
    WM_MOUSEWHEEL => {
      io.mouse_wheel += if GET_WHEEL_DELTA_WPARAM(w_param) > 0 {
        1.0f32
      } else {
        -1.0f32
      };
      TRUE
    },
    WM_MOUSEMOVE => {
      io.mouse_pos[0] = l_param as f32;
      io.mouse_pos[1] = l_param.shr(0x10) as f32;
      TRUE
    },
    WM_KEYDOWN | WM_SYSKEYDOWN => {
      if w_param < 0x100 {
        io.keys_down[w_param] = true;
      }
      TRUE
    },
    WM_KEYUP | WM_SYSKEYUP => {
      if w_param < 0x100 {
        io.keys_down[w_param] = false;
      }
      TRUE
    },
    WM_CHAR => {
      if w_param > 0 && w_param < 0x10000 {
        io.add_input_character(w_param as u8 as char);
      }
      TRUE
    },
    _ => FALSE
  };

  return if ret == TRUE {
    ret as LRESULT
  } else {
    CallWindowProcW(O_WND_PROC.unwrap(), hwnd, u_msg, w_param, l_param)
  };
}

type EndScene = extern "stdcall" fn(LPDIRECT3DDEVICE9) -> HRESULT;

pub extern "stdcall" fn hk_end_scene(p_device: LPDIRECT3DDEVICE9) -> HRESULT {
  // detour-rs doesn't play nice with unsafe "stdcall" functions, didn't dig deep, but appears to be a generic type limitation
  unsafe {
    if !INIT {
      IMGUI_RENDERER = Renderer::new(
        IMGUI_CONTEXT.as_mut().unwrap(),
        NonNull::new(p_device).expect("the directx device was null")
      );
      INIT = true;
    }


    let x = 15;
    let y = x;
    let width = 200;
    let height = width;
    let rect = D3DRECT {
      x1: x,
      y1: y,
      x2: x + width,
      y2: y + height
    };

    (*p_device).Clear(
      1, &rect as *const D3DRECT, D3DCLEAR_TARGET | D3DCLEAR_TARGET,
      D3DCOLOR_XRGB(0, 0xff, 0), 0f32, 0
    );

    let imgui = IMGUI_CONTEXT.as_mut().unwrap();
    imgui.io_mut().display_size = HWND_RECT.unwrap();
    imgui.io_mut().display_framebuffer_scale = [1.0f32, 1.0f32];
    imgui.io_mut().delta_time = 1.0f32 / 60.0f32;

    let ui = imgui.frame();
    let mut show = true;
    {
      // wnd_proc isn't working so this is really just good for rendering text and other static things
      Window::new(im_str!("Hello world"))
          .size([300.0, 600.0], imgui::Condition::FirstUseEver)
          .position([50.0, 50.0], imgui::Condition::FirstUseEver)
          .build(&ui, || {
            // Your window stuff here!
            ui.button(im_str!("Hi from this label!"), [100f32, 200f32]);
          });
      ui.show_demo_window(&mut show);
    }
    IMGUI_RENDERER.as_mut().unwrap().render(ui.render()).unwrap();

    return match &ENDSCENE_DETOUR {
      Ok(f) => f.call(p_device),
      Err(_) => 80004005, //E_FAIL
    };
  }
}

pub unsafe fn hook_device_functions(vtable: Vec<*const usize>) {
    ENDSCENE_DETOUR = GenericDetour::<EndScene>::new(
      mem::transmute(*vtable.get(42).unwrap()),
      hk_end_scene
    );
    IMGUI_CONTEXT = Some(Context::create());
    match &ENDSCENE_DETOUR {
      Ok(o) => {
        o.enable().unwrap();
      },
      _ => {},
    }
}

pub unsafe fn hook_wnd_proc(hwnd: HWND) {
  O_WND_PROC = Some(
    mem::transmute(SetWindowLongPtrW(hwnd, GWLP_WNDPROC, hk_wnd_proc as LONG_PTR))
  );
}