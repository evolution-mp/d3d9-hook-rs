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

type EndScene = extern "stdcall" fn(LPDIRECT3DDEVICE9) -> HRESULT;

static mut ENDSCENE_DETOUR: Result<GenericDetour<EndScene>, Error> = Err(Error::NotInitialized);

pub extern "stdcall" fn hk_end_scene(p_device: LPDIRECT3DDEVICE9) -> HRESULT {
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

  unsafe {
    (*p_device).Clear(
      1, &rect as *const D3DRECT, D3DCLEAR_TARGET | D3DCLEAR_TARGET,
      D3DCOLOR_XRGB(0, 0xff, 0), 0f32, 0
    );

    return match &ENDSCENE_DETOUR {
      Ok(f) => f.call(p_device),
      Err(_) => 80004005, //E_FAIL
    };
  }
}

pub fn hook_functions(vtable: Vec<*const usize>) {
  unsafe {
    ENDSCENE_DETOUR = GenericDetour::<extern "stdcall" fn(LPDIRECT3DDEVICE9) -> HRESULT>::new(
      mem::transmute(*vtable.get(42).unwrap()),
      hk_end_scene
    );
    println!("Created Endscene detour");
    match &ENDSCENE_DETOUR {
      Ok(o) => {
        o.enable().unwrap();
        println!("Endscene detour enabled");
      },
      _ => {},
    }
  }
}