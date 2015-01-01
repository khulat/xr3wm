use std::io::process::Command;
use std::thread::Thread;
use config::Config;
use layout::LayoutMsg;
use xlib_window_system::XlibWindowSystem;
use workspaces::{Workspaces, MoveOp};
use xlib::Window;

pub enum Cmd {
  Exec(String),
  SwitchWorkspace(uint),
  SwitchScreen(uint),
  MoveToWorkspace(uint),
  MoveToScreen(uint),
  SendLayoutMsg(LayoutMsg),
  KillClient,
  FocusUp,
  FocusDown,
  FocusMaster,
  SwapUp,
  SwapDown,
  SwapMaster,
}

impl Cmd {
  pub fn call<'a>(&self, ws: &XlibWindowSystem, workspaces: &mut Workspaces<'a>, config: &Config) {
    match *self {
      Cmd::Exec(ref cmd) => {
        debug!("Cmd::Exec: {}", cmd);
        exec(cmd.clone());
      },
      Cmd::SwitchWorkspace(index) => {
        debug!("Cmd::SwitchWorkspace: {}", index);
        workspaces.switch_to(ws, config, index - 1);
      },
      Cmd::SwitchScreen(screen) => {
        debug!("Cmd::SwitchScreen: {}", screen);
        workspaces.switch_to_screen(ws, config, screen - 1);
      },
      Cmd::MoveToWorkspace(index) => {
        debug!("Cmd::MoveToWorkspace: {}", index);
        workspaces.move_window_to(ws, config, index - 1);
      },
      Cmd::MoveToScreen(screen) => {
        debug!("Cmd::MoveToScreen: {}", screen);
        workspaces.move_window_to_screen(ws, config, screen - 1);
      },
      Cmd::SendLayoutMsg(ref msg) => {
        debug!("Cmd::SendLayoutMsg::{}", msg);
        workspaces.current_mut().get_layout_mut().deref_mut().send_msg(msg.clone());
        workspaces.current().redraw(ws, config);
      },
      Cmd::KillClient => {
        debug!("Cmd::KillClient: {}", workspaces.current_mut().focused_window());
        ws.kill_window(workspaces.current_mut().focused_window());
      },
      Cmd::FocusUp => {
        debug!("Cmd::FocusUp: {}", workspaces.current().focused_window());
        workspaces.current_mut().move_focus(ws, config, MoveOp::Up);
      },
      Cmd::FocusDown => {
        debug!("Cmd::FocusDown: {}", workspaces.current().focused_window());
        workspaces.current_mut().move_focus(ws, config, MoveOp::Down);
      },
      Cmd::FocusMaster => {
        debug!("Cmd::FocusMaster: {}", workspaces.current().focused_window());
        workspaces.current_mut().move_focus(ws, config, MoveOp::Swap);
      },
      Cmd::SwapUp => {
        debug!("Cmd::SwapUp: {}", workspaces.current().focused_window());
        workspaces.current_mut().move_window(ws, config, MoveOp::Up);
      },
      Cmd::SwapDown => {
        debug!("Cmd::SwapDown: {}", workspaces.current().focused_window());
        workspaces.current_mut().move_window(ws, config, MoveOp::Down);
      },
      Cmd::SwapMaster => {
        debug!("Cmd::SwapMaster: {}", workspaces.current().focused_window());
        workspaces.current_mut().move_window(ws, config, MoveOp::Swap);
      }
    }
  }
}

pub struct ManageHook {
  pub class_name: String,
  pub cmd: CmdManage
}

pub enum CmdManage {
  Move(uint),
  Float,
  Fullscreen,
  Ignore
}

impl CmdManage {
  pub fn call<'a>(&self, ws: &XlibWindowSystem, workspaces: &mut Workspaces<'a>, config: &Config, window: Window) {
    match *self {
      CmdManage::Move(index) => {
        debug!("CmdManage::Move: {}, {}", window, index - 1);
        workspaces.get_mut(index - 1).add_window(ws, config, window);
        workspaces.get_mut(index - 1).focus_window(ws, config, window);
      },
      CmdManage::Float => {
        debug!("CmdManage::Float");
        unimplemented!()
      },
      CmdManage::Fullscreen => {
        debug!("CmdManage::Fullscreen");
        unimplemented!()
      },
      CmdManage::Ignore => {
        debug!("CmdManage::Ignore");
        unimplemented!()
      }
    }
  }
}

pub enum LogInfo {
  Workspaces(Vec<String>, uint, Vec<uint>),
  Title(String),
  Layout(String)
}

pub struct LogHook<'a> {
  pub logs: Vec<CmdLogHook>,
  pub output: |Vec<LogInfo>|:'a -> String
}

impl<'a> LogHook<'a> {
  pub fn call<'b>(&mut self, ws: &XlibWindowSystem, workspaces: &Workspaces<'b>) {
    println!("{}", (self.output)(self.logs.iter().map(|x| x.call(ws, workspaces)).collect()));
  }
}

pub enum CmdLogHook {
  Workspaces,
  Title,
  Layout
}

impl CmdLogHook {
  pub fn call<'a>(&self, ws: &XlibWindowSystem, workspaces: &Workspaces<'a>) -> LogInfo {
    match *self {
      CmdLogHook::Workspaces => {
        LogInfo::Workspaces(workspaces.all().iter().map(|x| x.get_tag()).collect(), workspaces.get_index(), Vec::new())
      },
      CmdLogHook::Title => {
        LogInfo::Title(ws.get_window_title(workspaces.current().focused_window()))
      },
      CmdLogHook::Layout => {
        LogInfo::Layout(workspaces.current().get_layout().name())
      }
    }
  }
}

fn exec(cmd: String) {
  Thread::spawn(move || {
    let args : Vec<&str> = cmd.as_slice().split(' ').collect();

    if args.len() > 0 {
      let mut cmd = Command::new(args[0]);

      if args.len() > 1 {
        cmd.args(args.as_slice().slice_from(1));
      }

      match cmd.detached().output() {
        Ok(_) => (),
        _ => panic!("failed to start \"{}\"", cmd)
      }
    }
  }).detach();
}
