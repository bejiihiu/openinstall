use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command as SysCommand;
use std::rc::Rc;
use std::sync::mpsc;

use adw::prelude::*;
use gio::prelude::*;
use gtk4 as gtk;

use installer_core::runtime::Installer;
use installer_core::{
    Environment, InstallProgress, InstallStage, InstallationState, Manifest, VerificationOutcome,
};

use crate::i18n::{t, Locale};

type RefreshFn = Rc<RefCell<Option<Box<dyn Fn()>>>>;

enum ProgressUpdate {
    Progress(InstallProgress),
    VerifyResult(Result<VerificationOutcome, String>),
    InstallResult(Result<(String, PathBuf), String>),
    RemoveResult(Result<(String, PathBuf), String>),
    RollbackResult(Result<(String, PathBuf), String>),
}

struct UiData {
    locale: Locale,
    manifest: Option<Manifest>,
    manifest_path: Option<PathBuf>,
    environment: Environment,
    installer: Installer,
    window: Option<adw::ApplicationWindow>,
    page: Page,
    verification: Option<VerificationOutcome>,
    install_state: Option<InstallationState>,
    staged_path: Option<PathBuf>,
    latest_progress: Option<InstallProgress>,
    toast_overlay: Option<adw::ToastOverlay>,
    progress_bar: Option<gtk::ProgressBar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Manifest,
    Installing,
    Done,
    Error(String),
}

fn ensure_gtk_renderer() {
    if std::env::var("GSK_RENDERER").is_err() {
        std::env::set_var("GSK_RENDERER", "cairo");
    }
    if std::env::var("GDK_DISABLE").is_err() {
        std::env::set_var("GDK_DISABLE", "vulkan");
    }
}

pub fn run(args: Vec<String>) {
    ensure_gtk_renderer();
    let application = adw::Application::builder()
        .application_id("io.openinstall.installer")
        .flags(adw::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    let args_activate = args.clone();
    let args_open = args;
    application.connect_activate(move |app| {
        let locale = Locale::detect();
        let manifest_path = args_activate.first().map(PathBuf::from);
        let environment = Environment::detect();
        let installer = Installer::default();

        let (manifest, _load_error) = match manifest_path.as_ref() {
            Some(path) => match Manifest::from_path(path) {
                Ok(m) => (Some(m), None),
                Err(e) => (None, Some(e.to_string())),
            },
            None => (Some(demo_manifest()), None),
        };

        let install_state = manifest
            .as_ref()
            .and_then(|m| installer.inspect(m, &environment).ok());

        let data = Rc::new(RefCell::new(UiData {
            locale,
            manifest,
            manifest_path,
            environment,
            installer,
            window: None,
            page: Page::Manifest,
            verification: None,
            install_state,
            staged_path: None,
            latest_progress: None,
            toast_overlay: None,
            progress_bar: None,
        }));

        build_window(app, data);
    });

    application.connect_open(move |_app, _files, _hint| {
        let locale = Locale::detect();
        let environment = Environment::detect();
        let installer = Installer::default();

        let manifest_path = args_open.first().map(PathBuf::from);
        let (manifest, _load_error): (Option<Manifest>, Option<String>) =
            match manifest_path.as_ref() {
                Some(path) => match Manifest::from_path(path) {
                    Ok(m) => (Some(m), None),
                    Err(_) => (Some(demo_manifest()), None),
                },
                None => (Some(demo_manifest()), None),
            };

        let install_state = manifest
            .as_ref()
            .and_then(|m| installer.inspect(m, &environment).ok());

        let data = Rc::new(RefCell::new(UiData {
            locale,
            manifest,
            manifest_path,
            environment,
            installer,
            window: None,
            page: Page::Manifest,
            verification: None,
            install_state,
            staged_path: None,
            latest_progress: None,
            toast_overlay: None,
            progress_bar: None,
        }));

        let app = _app.clone();
        build_window(&app, data);
    });

    application.run();
}

fn build_window(app: &adw::Application, data: Rc<RefCell<UiData>>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(600)
        .default_height(600)
        .build();

    let toast_overlay = adw::ToastOverlay::new();

    let toolbar = adw::ToolbarView::new();

    let header = adw::HeaderBar::new();

    let subtitle = data.borrow().manifest.as_ref().map(|m| m.publisher.clone());
    let title_widget = adw::WindowTitle::new(
        t(data.borrow().locale, "app.title"),
        subtitle.as_deref().unwrap_or(""),
    );
    header.set_title_widget(Some(&title_widget));

    let menu = adw::gio::Menu::new();
    let section1 = adw::gio::Menu::new();
    section1.append(
        Some(&t(data.borrow().locale, "menu.verify")),
        Some("app.verify"),
    );
    section1.append(
        Some(&t(data.borrow().locale, "menu.rollback")),
        Some("app.rollback"),
    );
    section1.append(
        Some(&t(data.borrow().locale, "menu.reload")),
        Some("app.reload"),
    );
    menu.append_section(None, &section1);

    let section2 = adw::gio::Menu::new();
    section2.append(
        Some(&t(data.borrow().locale, "menu.cache_info")),
        Some("app.cache-info"),
    );
    section2.append(
        Some(&t(data.borrow().locale, "menu.history")),
        Some("app.history"),
    );
    menu.append_section(None, &section2);

    let section3 = adw::gio::Menu::new();
    section3.append(
        Some(&t(data.borrow().locale, "menu.about")),
        Some("app.about"),
    );
    menu.append_section(None, &section3);

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build();

    header.pack_end(&menu_button);
    toolbar.add_top_bar(&header);

    let progress_bar = gtk::ProgressBar::new();
    progress_bar.add_css_class("osd");
    progress_bar.set_visible(false);
    toolbar.add_top_bar(&progress_bar);

    let page_content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page_content.set_hexpand(true);
    page_content.set_vexpand(true);
    toast_overlay.set_child(Some(&page_content));
    toolbar.set_content(Some(&toast_overlay));

    window.set_content(Some(&toolbar));

    data.borrow_mut().window = Some(window.clone());
    data.borrow_mut().toast_overlay = Some(toast_overlay.clone());
    data.borrow_mut().progress_bar = Some(progress_bar.clone());

    let (tx, rx) = mpsc::channel::<ProgressUpdate>();

    let refresh: RefreshFn = Rc::new(RefCell::new(None));

    let data_c = Rc::clone(&data);
    let tx_c = tx.clone();
    let pc = page_content.clone();

    *refresh.borrow_mut() = Some(Box::new(move || {
        clear_children(&pc);
        let current = data_c.borrow();
        let page = current.page.clone();
        drop(current);
        match page {
            Page::Manifest => render_manifest(&data_c, &pc, &tx_c),
            Page::Installing => render_installing(&data_c, &pc),
            Page::Done => render_done(&data_c, &pc),
            Page::Error(msg) => render_error(&data_c, &pc, msg),
        }
    }));

    let action_group = adw::gio::SimpleActionGroup::new();
    window.add_action_group("app", &action_group);

    let data_v = Rc::clone(&data);
    let tx_v = tx.clone();
    let action = adw::gio::SimpleAction::new("verify", None);
    action.connect_activate(move |_, _| {
        let manifest = data_v.borrow().manifest.clone();
        let env = data_v.borrow().environment.clone();
        let installer = data_v.borrow().installer.clone();
        let tx = tx_v.clone();
        std::thread::spawn(move || {
            let m = match manifest {
                Some(ref m) => m.clone(),
                None => return,
            };
            let result = installer.verify(&m, &env).map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::VerifyResult(result));
        });
    });
    action_group.add_action(&action);

    let data_r = Rc::clone(&data);
    let tx_r = tx.clone();
    let action = adw::gio::SimpleAction::new("rollback", None);
    action.connect_activate(move |_, _| {
        let mut d = data_r.borrow_mut();
        d.page = Page::Installing;
        let manifest = d.manifest.clone();
        let env = d.environment.clone();
        let installer = d.installer.clone();
        let tx = tx_r.clone();
        drop(d);
        std::thread::spawn(move || {
            let m = match manifest {
                Some(ref m) => m.clone(),
                None => return,
            };
            let result = installer
                .rollback(&m, &env)
                .map(|o| (o.command, o.staged_path))
                .map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::RollbackResult(result));
        });
    });
    action_group.add_action(&action);

    let data_rl = Rc::clone(&data);
    let refresh_rl = Rc::clone(&refresh);
    let action = adw::gio::SimpleAction::new("reload", None);
    action.connect_activate(move |_, _| {
        let mut d = data_rl.borrow_mut();
        let path = d.manifest_path.clone();
        let env = Environment::detect();
        let installer = Installer::default();
        let (manifest, _) = match path.as_ref() {
            Some(p) => match Manifest::from_path(p) {
                Ok(m) => (Some(m), None::<()>),
                Err(_) => (None, None),
            },
            None => (Some(demo_manifest()), None),
        };
        let install_state = manifest
            .as_ref()
            .and_then(|m| installer.inspect(m, &env).ok());
        d.environment = env;
        d.installer = installer;
        d.manifest = manifest;
        d.install_state = install_state;
        d.verification = None;
        d.staged_path = None;
        drop(d);
        if let Some(ref f) = *refresh_rl.borrow() {
            f();
        }
    });
    action_group.add_action(&action);

    let data_ci = Rc::clone(&data);
    let action = adw::gio::SimpleAction::new("cache-info", None);
    action.connect_activate(move |_, _| {
        let installer = data_ci.borrow().installer.clone();
        let result = installer
            .cache_info()
            .map(|info| format!("{} files, {} bytes", info.file_count, info.total_bytes))
            .unwrap_or_else(|e| e.to_string());
        if let Some(ref ov) = data_ci.borrow().toast_overlay {
            let toast = adw::Toast::new(&result);
            ov.add_toast(toast);
        }
    });
    action_group.add_action(&action);

    let data_h = Rc::clone(&data);
    let action = adw::gio::SimpleAction::new("history", None);
    action.connect_activate(move |_, _| {
        let installer = data_h.borrow().installer.clone();
        let result = installer
            .get_history()
            .map(|entries| {
                if entries.is_empty() {
                    "No history".to_string()
                } else {
                    entries
                        .iter()
                        .map(|e| {
                            format!(
                                "{} v{} via {} — {}",
                                e.package_id,
                                e.version,
                                e.package_manager,
                                e.installed_at_unix_secs
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            })
            .unwrap_or_else(|e| e.to_string());
        if let Some(ref ov) = data_h.borrow().toast_overlay {
            let toast = adw::Toast::new(&result);
            ov.add_toast(toast);
        }
    });
    action_group.add_action(&action);

    let window_clone = window.clone();
    let action = adw::gio::SimpleAction::new("about", None);
    action.connect_activate(move |_, _| {
        let about = adw::AboutDialog::builder()
            .application_name("OpenInstall")
            .application_icon("system-software-install-symbolic")
            .version(env!("CARGO_PKG_VERSION"))
            .developer_name("OpenInstall Contributors")
            .developers(vec!["OpenInstall Contributors".to_string()])
            .copyright("© 2025 OpenInstall Contributors")
            .license_type(gtk::License::MitX11)
            .website("https://github.com/bejiihiu/openinstall")
            .issue_url("https://github.com/bejiihiu/openinstall/issues")
            .build();
        about.present(Some(&window_clone));
    });
    action_group.add_action(&action);

    let data_clone = Rc::clone(&data);
    let refresh_clone = Rc::clone(&refresh);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        while let Ok(msg) = rx.try_recv() {
            let mut d = data_clone.borrow_mut();
            match msg {
                ProgressUpdate::Progress(p) => {
                    d.page = Page::Installing;
                    d.latest_progress = Some(p);
                }
                ProgressUpdate::VerifyResult(result) => match result {
                    Ok(outcome) => {
                        if let Some(ref ov) = d.toast_overlay {
                            let toast = adw::Toast::new(t(d.locale, "toast.verified"));
                            ov.add_toast(toast);
                        }
                        d.verification = Some(outcome);
                    }
                    Err(e) => d.page = Page::Error(e),
                },
                ProgressUpdate::InstallResult(result) => match result {
                    Ok((_cmd, staged)) => {
                        d.staged_path = Some(staged);
                        d.page = Page::Done;
                        d.install_state = None;
                    }
                    Err(e) => d.page = Page::Error(e),
                },
                ProgressUpdate::RemoveResult(result) => match result {
                    Ok(_) => {
                        d.staged_path = None;
                        d.page = Page::Done;
                        d.install_state = Some(InstallationState::NotInstalled);
                    }
                    Err(e) => d.page = Page::Error(e),
                },
                ProgressUpdate::RollbackResult(result) => match result {
                    Ok((_cmd, staged)) => {
                        d.staged_path = Some(staged);
                        d.page = Page::Done;
                        d.install_state = None;
                    }
                    Err(e) => d.page = Page::Error(e),
                },
            }
            drop(d);
            if let Some(ref f) = *refresh_clone.borrow() {
                f();
            }
        }
        glib::ControlFlow::Continue
    });

    if let Some(ref f) = *refresh.borrow() {
        f();
    }
    window.present();
}

fn render_manifest(
    data: &Rc<RefCell<UiData>>,
    parent: &gtk::Box,
    tx: &mpsc::Sender<ProgressUpdate>,
) {
    let locale = data.borrow().locale;
    let manifest = match &data.borrow().manifest {
        Some(m) => m.clone(),
        None => return,
    };
    let env = data.borrow().environment.clone();
    let install_state = data.borrow().install_state.clone();
    let verification = data.borrow().verification.clone();

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hexpand(true);

    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .tightening_threshold(400)
        .build();

    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    if verification
        .as_ref()
        .is_some_and(|v| v.signature_ok == Some(false))
    {
        let banner = adw::Banner::builder()
            .title(t(locale, "manifest.signature_warning"))
            .build();
        banner.set_css_classes(&["destructive"]);
        content.append(&banner);
    }

    let icon_name = manifest
        .image
        .as_deref()
        .unwrap_or("system-software-install-symbolic");
    let desc = if manifest.description.is_empty() {
        &manifest.publisher
    } else {
        &manifest.description
    };
    let status_page = adw::StatusPage::builder()
        .icon_name(icon_name)
        .title(&manifest.name)
        .description(desc)
        .build();
    content.append(&status_page);

    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let details_group = adw::PreferencesGroup::builder()
        .title(t(locale, "status.details"))
        .build();

    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.publisher"))
        .subtitle(&manifest.publisher)
        .build();
    details_group.add(&row);

    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.version"))
        .subtitle(&manifest.version)
        .build();
    details_group.add(&row);

    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.license"))
        .subtitle(
            manifest
                .license
                .as_deref()
                .unwrap_or(t(locale, "detail.not_set")),
        )
        .build();
    details_group.add(&row);

    let homepage_val = manifest
        .homepage
        .as_deref()
        .unwrap_or(t(locale, "detail.not_set"));
    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.homepage"))
        .subtitle(homepage_val)
        .activatable(true)
        .build();
    if let Some(ref url) = manifest.homepage {
        let url = url.clone();
        row.connect_activated(move |_| {
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        });
    }
    details_group.add(&row);

    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.architecture"))
        .subtitle(&env.architecture)
        .build();
    details_group.add(&row);

    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.distribution"))
        .subtitle(&env.distro)
        .build();
    details_group.add(&row);

    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.package_manager"))
        .subtitle(&env.package_manager.to_string())
        .build();
    details_group.add(&row);

    content.append(&details_group);

    let security_group = adw::PreferencesGroup::builder()
        .title(t(locale, "status.security"))
        .build();

    let integrity = if manifest.sha256.is_some() {
        t(locale, "detail.available")
    } else {
        t(locale, "detail.not_set")
    };
    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.integrity"))
        .subtitle(integrity)
        .build();
    security_group.add(&row);

    let sig_status = match &verification {
        Some(v) => {
            if v.signature_ok == Some(true) {
                t(locale, "detail.valid")
            } else if v.signature_ok == Some(false) {
                t(locale, "detail.invalid")
            } else {
                t(locale, "detail.not_provided")
            }
        }
        None => {
            if manifest.signature.is_some() {
                t(locale, "detail.available")
            } else {
                t(locale, "detail.not_provided")
            }
        }
    };
    let row = adw::ActionRow::builder()
        .title(t(locale, "detail.signature"))
        .subtitle(sig_status)
        .build();
    security_group.add(&row);

    content.append(&security_group);

    if install_state.is_some() {
        let status_group = adw::PreferencesGroup::builder()
            .title(t(locale, "status.status"))
            .build();

        let state_text = match &install_state {
            Some(InstallationState::NotInstalled) => t(locale, "state.not_installed").to_string(),
            Some(InstallationState::SameVersion { version }) => {
                format!("{} ({})", t(locale, "state.same_version"), version)
            }
            Some(InstallationState::DifferentVersion {
                current_version,
                available_version,
            }) => {
                format!(
                    "{}: {} \u{2192} {}",
                    t(locale, "state.update_available"),
                    current_version,
                    available_version,
                )
            }
            None => String::new(),
        };
        let row = adw::ActionRow::builder()
            .title(t(locale, "status.status"))
            .subtitle(&state_text)
            .build();
        status_group.add(&row);
        content.append(&status_group);
    }

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    button_box.set_halign(gtk::Align::Center);
    button_box.set_margin_top(12);

    let install_btn = gtk::Button::builder()
        .label(match &install_state {
            Some(InstallationState::NotInstalled) | None => t(locale, "manifest.install"),
            Some(InstallationState::SameVersion { .. }) => t(locale, "manifest.reinstall"),
            Some(InstallationState::DifferentVersion { .. }) => t(locale, "manifest.update"),
        })
        .css_classes(["suggested-action", "pill"])
        .build();

    let remove_btn = gtk::Button::builder()
        .label(t(locale, "manifest.remove"))
        .css_classes(["destructive-action", "pill"])
        .build();

    let has_pkg = manifest.package_for_environment(&env).is_some();
    let bad_sig = verification
        .as_ref()
        .is_some_and(|v| v.signature_ok == Some(false));
    install_btn.set_sensitive(has_pkg && !bad_sig);
    remove_btn.set_sensitive(has_pkg);

    button_box.append(&install_btn);
    button_box.append(&remove_btn);
    content.append(&button_box);

    clamp.set_child(Some(&content));
    scroll.set_child(Some(&clamp));
    parent.append(&scroll);

    let data_install = Rc::clone(data);
    let tx_install = tx.clone();
    install_btn.connect_clicked(move |_| {
        let mut d = data_install.borrow_mut();
        d.page = Page::Installing;
        let manifest = d.manifest.clone();
        let env = d.environment.clone();
        let installer = d.installer.clone();
        let tx = tx_install.clone();
        drop(d);
        std::thread::spawn(move || {
            let m = match manifest {
                Some(ref m) => m.clone(),
                None => return,
            };
            let (progress_tx, progress_rx) = mpsc::channel::<InstallProgress>();
            let tx_fwd = tx.clone();
            std::thread::spawn(move || {
                while let Ok(p) = progress_rx.recv() {
                    if tx_fwd.send(ProgressUpdate::Progress(p)).is_err() {
                        break;
                    }
                }
            });
            let result = installer
                .install_with_progress(&m, &env, progress_tx)
                .map(|o| (o.command, o.staged_path))
                .map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::InstallResult(result));
        });
    });

    let data_remove = Rc::clone(data);
    let tx_remove = tx.clone();
    remove_btn.connect_clicked(move |_| {
        let mut d = data_remove.borrow_mut();
        d.page = Page::Installing;
        let manifest = d.manifest.clone();
        let env = d.environment.clone();
        let installer = d.installer.clone();
        let tx = tx_remove.clone();
        drop(d);
        std::thread::spawn(move || {
            let m = match manifest {
                Some(ref m) => m.clone(),
                None => return,
            };
            let result = installer
                .remove(&m, &env)
                .map(|o| (o.command, o.staged_path))
                .map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::RemoveResult(result));
        });
    });
}

fn render_installing(data: &Rc<RefCell<UiData>>, parent: &gtk::Box) {
    let locale = data.borrow().locale;

    let status_page = adw::StatusPage::builder()
        .icon_name("emblem-synchronizing-symbolic")
        .title(t(locale, "install.progress"))
        .build();

    let spinner = gtk::Spinner::new();
    spinner.set_size_request(48, 48);
    spinner.set_visible(true);
    spinner.start();
    status_page.set_child(Some(&spinner));

    let stored = data.borrow().latest_progress.clone();
    let mut desc = t(locale, "install.downloaded").to_string();
    if let Some(ref p) = stored {
        if p.total_bytes > 0 {
            desc = format!("{} / {} bytes", p.downloaded_bytes, p.total_bytes);
            if let Some(ref pb) = data.borrow().progress_bar {
                let frac = p.downloaded_bytes as f64 / p.total_bytes as f64;
                pb.set_fraction(frac.clamp(0.0, 1.0));
                pb.set_visible(true);
            }
        } else if let Some(ref pb) = data.borrow().progress_bar {
            pb.set_pulse_step(0.05);
            pb.pulse();
            pb.set_visible(true);
        }
        if p.stage == InstallStage::Done {
            desc = t(locale, "install.progress").to_string();
        }
    }
    status_page.set_description(Some(&desc));

    parent.append(&status_page);
}

fn render_done(data: &Rc<RefCell<UiData>>, parent: &gtk::Box) {
    let locale = data.borrow().locale;

    let status_page = adw::StatusPage::builder()
        .icon_name("object-select-symbolic")
        .title(t(locale, "done.title"))
        .description(t(locale, "done.success_install"))
        .build();

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    btn_box.set_halign(gtk::Align::Center);

    let app_name = data.borrow().manifest.as_ref().map(|m| m.name.clone());
    let launch_name = app_name.unwrap_or_default();
    let launch_btn = gtk::Button::builder()
        .label(t(locale, "done.launch"))
        .css_classes(["suggested-action", "pill"])
        .build();
    launch_btn.connect_clicked(move |_| {
        let safe_name = launch_name.replace('/', "_");
        let _ = SysCommand::new(&safe_name).spawn();
    });
    btn_box.append(&launch_btn);

    let close_btn = gtk::Button::builder()
        .label(t(locale, "done.close"))
        .css_classes(["pill"])
        .build();
    let data_close = Rc::clone(data);
    close_btn.connect_clicked(move |_| {
        if let Some(ref window) = data_close.borrow().window {
            window.close();
        }
    });
    btn_box.append(&close_btn);

    status_page.set_child(Some(&btn_box));
    parent.append(&status_page);
}

fn render_error(data: &Rc<RefCell<UiData>>, parent: &gtk::Box, message: String) {
    let locale = data.borrow().locale;

    let status_page = adw::StatusPage::builder()
        .icon_name("dialog-error-symbolic")
        .title(t(locale, "error.title"))
        .description(&message)
        .build();

    let close_btn = gtk::Button::builder()
        .label(t(locale, "error.close"))
        .css_classes(["pill"])
        .build();
    let data_close = Rc::clone(data);
    close_btn.connect_clicked(move |_| {
        if let Some(ref window) = data_close.borrow().window {
            window.close();
        }
    });

    status_page.set_child(Some(&close_btn));
    parent.append(&status_page);
}

fn clear_children(box_: &gtk::Box) {
    while let Some(child) = box_.first_child() {
        box_.remove(&child);
    }
}

fn demo_manifest() -> Manifest {
    Manifest {
        name: "OpenInstall Demo".to_string(),
        publisher: "OpenInstall".to_string(),
        version: "0.1.0".to_string(),
        description: "A cross-distro Linux application installer.".to_string(),
        homepage: Some("https://example.com".to_string()),
        license: Some("MIT".to_string()),
        changelog: Some("Initial release".to_string()),
        image: None,
        packages: installer_core::PackageMatrix {
            arch: Some("https://example.com/demo.pkg.tar.zst".to_string()),
            ubuntu: Some("https://example.com/demo.deb".to_string()),
            fedora: Some("https://example.com/demo.rpm".to_string()),
            opensuse: Some("https://example.com/demo.rpm".to_string()),
            flatpak: None,
            appimage: Some("https://example.com/demo.AppImage".to_string()),
            windows: None,
            macos: None,
        },
        sha256: None,
        signature: None,
        scripts: None,
    }
}
