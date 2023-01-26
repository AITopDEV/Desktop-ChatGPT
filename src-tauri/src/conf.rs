use log::{error, info};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};
use tauri::{Manager, Theme};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

use crate::utils::{app_root, create_file, exists};

pub const ISSUES_URL: &str = "https://github.com/lencx/ChatGPT/issues";
pub const UPDATE_LOG_URL: &str = "https://github.com/lencx/ChatGPT/blob/main/UPDATE_LOG.md";
pub const AWESOME_URL: &str = "https://github.com/lencx/ChatGPT/blob/main/AWESOME.md";
pub const BUY_COFFEE: &str = "https://www.buymeacoffee.com/lencx";
pub const GITHUB_PROMPTS_CSV_URL: &str =
  "https://raw.githubusercontent.com/f/awesome-chatgpt-prompts/main/prompts.csv";

pub const APP_CONF_PATH: &str = "chat.conf.json";
pub const CHATGPT_URL: &str = "https://chat.openai.com";
pub const UA_MOBILE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

macro_rules! pub_struct {
  ($name:ident {$($field:ident: $t:ty,)*}) => {
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct $name {
      $(pub $field: $t),*
    }
  }
}

pub_struct!(AppConf {
  titlebar: bool,
  hide_dock_icon: bool,
  // macOS and Windows: Light / Dark / System
  theme: String,
  // auto update policy: Prompt / Silent / Disable
  auto_update: String,
  tray: bool,
  popup_search: bool,
  stay_on_top: bool,
  main_dashboard: bool,
  tray_dashboard: bool,
  main_origin: String,
  tray_origin: String,
  default_origin: String,
  ua_window: String,
  ua_tray: String,
  global_shortcut: Option<String>,
});

impl AppConf {
  pub fn new() -> Self {
    info!("conf_init");
    Self {
      titlebar: !cfg!(target_os = "macos"),
      hide_dock_icon: false,
      theme: "Light".into(),
      auto_update: "Prompt".into(),
      tray: true,
      popup_search: false,
      stay_on_top: false,
      main_dashboard: false,
      tray_dashboard: false,
      main_origin: CHATGPT_URL.into(),
      tray_origin: CHATGPT_URL.into(),
      default_origin: CHATGPT_URL.into(),
      ua_tray: UA_MOBILE.into(),
      ua_window: "".into(),
      global_shortcut: None,
    }
  }

  pub fn file_path() -> PathBuf {
    app_root().join(APP_CONF_PATH)
  }

  pub fn read() -> Self {
    match std::fs::read_to_string(Self::file_path()) {
      Ok(v) => serde_json::from_str::<AppConf>(&v).unwrap(),
      Err(err) => {
        error!("conf_read_error: {}", err);
        Self::default()
      }
    }
  }

  pub fn write(self) -> Self {
    let path = &Self::file_path();
    if !exists(path) {
      create_file(path).unwrap();
      info!("conf_init")
    }
    if let Ok(v) = serde_json::to_string_pretty(&self) {
      std::fs::write(path, v).unwrap_or_else(|err| {
        error!("conf_write_error: {}", err);
        Self::default().write();
      });
    } else {
      error!("conf_ser_error");
    }
    self
  }

  pub fn amend(self, json: Value) -> Self {
    let val = serde_json::to_value(&self).unwrap();
    let mut config: BTreeMap<String, Value> = serde_json::from_value(val).unwrap();
    let new_json: BTreeMap<String, Value> = serde_json::from_value(json).unwrap();

    for (k, v) in new_json {
      config.insert(k, v);
    }

    match serde_json::to_string_pretty(&config) {
      Ok(v) => match serde_json::from_str::<AppConf>(&v) {
        Ok(v) => v,
        Err(err) => {
          error!("conf_amend_parse_error: {}", err);
          self
        }
      },
      Err(err) => {
        error!("conf_amend_str_error: {}", err);
        self
      }
    }
  }

  #[cfg(target_os = "macos")]
  pub fn titlebar(self) -> TitleBarStyle {
    if self.titlebar {
      TitleBarStyle::Transparent
    } else {
      TitleBarStyle::Overlay
    }
  }

  pub fn theme_mode() -> Theme {
    match cmd::get_theme().to_lowercase().as_str() {
      "system" => match dark_light::detect() {
        // Dark mode
        dark_light::Mode::Dark => Theme::Dark,
        // Light mode
        dark_light::Mode::Light => Theme::Light,
        // Unspecified
        dark_light::Mode::Default => Theme::Light,
      },
      "dark" => Theme::Dark,
      _ => Theme::Light,
    }
  }

  pub fn theme_check(self, mode: &str) -> bool {
    self.theme.to_lowercase() == mode
  }

  pub fn restart(self, app: tauri::AppHandle) {
    tauri::api::process::restart(&app.env());
  }
}

impl Default for AppConf {
  fn default() -> Self {
    Self::new()
  }
}

pub mod cmd {
  use super::AppConf;
  use tauri::{command, AppHandle, Manager};

  #[command]
  pub fn get_app_conf() -> AppConf {
    AppConf::read()
  }

  #[command]
  pub fn reset_app_conf() -> AppConf {
    AppConf::default().write()
  }

  #[command]
  pub fn get_theme() -> String {
    AppConf::read().theme
  }

  #[command]
  pub fn form_confirm(_app: AppHandle, data: serde_json::Value) {
    AppConf::read().amend(serde_json::json!(data)).write();
  }

  #[command]
  pub fn form_cancel(app: AppHandle, label: &str, title: &str, msg: &str) {
    let win = app.app_handle().get_window(label).unwrap();
    tauri::api::dialog::ask(
      app.app_handle().get_window(label).as_ref(),
      title,
      msg,
      move |is_cancel| {
        if is_cancel {
          win.close().unwrap();
        }
      },
    );
  }

  #[command]
  pub fn form_msg(app: AppHandle, label: &str, title: &str, msg: &str) {
    let win = app.app_handle().get_window(label);
    tauri::api::dialog::message(win.as_ref(), title, msg);
  }
}
