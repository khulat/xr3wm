#![allow(non_upper_case_globals)]
extern crate libc;

use keycode::{MOD_2, Keystroke};
use layout::Rect;
use std::ptr::null_mut;
use std::mem::{uninitialized, transmute};
use std::str::raw::c_str_to_static_slice;
use self::libc::{c_void, c_int, c_char};
use self::libc::funcs::c95::stdlib::malloc;
use self::XlibEvent::*;
use xlib::*;

extern fn error_handler(display: *mut Display, event: *mut XErrorEvent) -> c_int {
  // TODO: proper error handling
  // HACK: fixes LeaveNotify on invalid windows
  return 0;
}

const KeyPress               : i32 = 2;
const EnterNotify            : i32 = 7;
const FocusOut               : i32 = 10;
const Destroy                : i32 = 17;
const MapRequest             : i32 = 20;
const ConfigureRequest       : i32 = 23;

pub struct XlibWindowSystem {
  display:   *mut Display,
  root:      Window,
  event:     *mut c_void
}

pub enum XlibEvent {
  XMapRequest(Window),
  XConfigurationRequest(Window, WindowChanges, u64),
  XDestroy(Window),
  XEnterNotify(Window),
  XFocusOut(Window),
  XKeyPress(Window, Keystroke),
  Ignored
}

pub struct WindowChanges {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub border_width: u32,
  pub sibling: Window,
  pub stack_mode: u32,
}

impl XlibWindowSystem {
  pub fn new() -> Option<XlibWindowSystem> {
    unsafe {
      let display = XOpenDisplay(null_mut());
      if display.is_null() {
        return None;
      }

      let root = XDefaultRootWindow(display);
      XSelectInput(display, root, 0x1A0035);

      XSetErrorHandler(error_handler as *mut u8);

      Some(XlibWindowSystem{
        display: display,
        root: root,
        event: malloc(256)
      })
    }
  }

  pub fn setup_window(&self, x: u32, y: u32, width: u32, height: u32, border_width: u32, border_color: u64, window: Window) {
    self.set_window_border_width(window, border_width);
    self.set_window_border_color(window, border_color);
    self.move_resize_window(window, x, y, width - (2 * border_width), height - (2 * border_width));
  }

  pub fn configure_window(&mut self, window: Window, window_changes: WindowChanges, mask: u64) {
    unsafe {
      let mut ret_window_changes = XWindowChanges{
        x: window_changes.x as i32,
        y: window_changes.y as i32,
        width: window_changes.width as i32,
        height: window_changes.height as i32,
        border_width: window_changes.border_width as i32,
        sibling: window_changes.sibling,
        stack_mode: window_changes.stack_mode as i32
      };
      XConfigureWindow(self.display, window, mask as u32, &mut ret_window_changes);
    }
  }

  pub fn new_vroot(&self) -> Window {
    unsafe {
      let window = XCreateSimpleWindow(self.display, self.root, 0, 0, self.get_display_width(0), self.get_display_height(0), 0, 0, 0);
      XMapWindow(self.display, window);
      window
    }
  }

  pub fn map_to_parent(&self, parent: Window, window: Window) {
    unsafe {
      XReparentWindow(self.display, window, parent, 0, 0);
      XMapWindow(self.display, window);
    }
  }

  pub fn move_resize_window(&self, window: Window, x: u32, y: u32, width: u32, height: u32) {
    unsafe {
      XMoveResizeWindow(self.display, window, x as i32, y as i32, width, height);
    }
  }

  pub fn raise_window(&self, window: Window) {
    unsafe {
      XRaiseWindow(self.display, window);
    }
  }

  pub fn focus_window(&self, window: Window, color: u64) {
    unsafe {
      XSetInputFocus(self.display, window, 1, 0);
      XSetWindowBorder(self.display, window, color);
    }
  }

  pub fn kill_window(&self, window: Window) {
    unsafe {
      XKillClient(self.display, window);
    }
  }

  pub fn sync(&self) {
    unsafe {
      XSync(self.display, 1);
    }
  }

  pub fn grab_modifier(&self, mod_key: u8) {
    unsafe {
      XGrabKey(self.display, 0, mod_key as u32, self.root, 1, 0, 1);
      XGrabKey(self.display, 0, (mod_key | MOD_2) as u32, self.root, 1, 0, 1 );
    }
  }

  pub fn keycode_to_string(&self, keycode: u32) -> String {
    unsafe {
      let keysym = XKeycodeToKeysym(self.display, keycode as u8, 0);
      String::from_str(c_str_to_static_slice(transmute(XKeysymToString(keysym))))
    }
  }

  pub fn set_window_border_width(&self, window: Window, width: u32) {
    if window != self.root {
      unsafe {
        XSetWindowBorderWidth(self.display, window, width);
      }
    }
  }

  pub fn set_window_border_color(&self, window: Window, color: u64) {
    if window != self.root {
      unsafe {
        XSetWindowBorder(self.display, window, color);
      }
    }
  }

  pub fn get_display_width(&self, screen: u32) -> u32 {
    unsafe {
      XDisplayWidth(self.display, screen as i32) as u32
    }
  }

  pub fn get_display_height(&self, screen: u32) -> u32 {
    unsafe {
      XDisplayHeight(self.display, screen as i32) as u32
    }
  }

  pub fn get_display_rect(&self, screen: u32) -> Rect {
    Rect{x: 0, y: 0, width: self.get_display_width(screen), height: self.get_display_height(screen)}
  }

  pub fn get_window_name(&self, window: Window) -> String {
    if window == self.root {
      return String::from_str("root");
    }

    unsafe {
      let mut name : *mut c_char = uninitialized();
      XFetchName(self.display, window, &mut name);
      String::from_str(c_str_to_static_slice(transmute(name)))
    }
  }

  fn cast_event_to<T>(&self) -> &T {
    unsafe {
      &*(self.event as *const T)
    }
  }

  pub fn get_event(&self) -> XlibEvent {
    unsafe {
      XNextEvent(self.display, self.event);
    }

    let evt_type : c_int = *self.cast_event_to();
    match evt_type{
      MapRequest => {
        let evt : &XMapRequestEvent = self.cast_event_to();
        unsafe {
          XSelectInput(self.display, evt.window, 0x420030);
        }
        
        XMapRequest(evt.window)
      },
      ConfigureRequest => {
        let event : &XConfigureRequestEvent = self.cast_event_to();
        let changes = WindowChanges{
          x: event.x as u32,
          y: event.y as u32,
          width: event.width as u32,
          height: event.height as u32,
          border_width: event.border_width as u32,
          sibling: event.above as Window,
          stack_mode: event.detail as u32
        };
        XConfigurationRequest(event.window, changes, event.value_mask)
      },
      Destroy => {
        let evt : &XDestroyWindowEvent = self.cast_event_to();
        XDestroy(evt.window)
      },
      EnterNotify => {
        let evt : &XEnterWindowEvent = self.cast_event_to();
        if evt.detail != 2 {
          XEnterNotify(evt.window)
        } else {
          Ignored
        }
      },
      FocusOut => {
        let evt : &XFocusOutEvent = self.cast_event_to();
        if evt.detail != 5 {
          XFocusOut(evt.window)
        } else {
          Ignored
        }
      },
      KeyPress => {
        let evt : &XKeyPressedEvent = self.cast_event_to();
        XKeyPress(evt.window, Keystroke{mods: evt.state as u8, key: self.keycode_to_string(evt.keycode)})
      },
      _ => {
        Ignored
      }
    }
  }
}