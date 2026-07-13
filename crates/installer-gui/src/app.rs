use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command as SysCommand;
use std::rc::Rc;
use std::sync::mpsc;

use adw::prelude::*;
use gtk4 as gtk;

use installer_core::runtime::Installer;
use installer_core::{
    Environment, InstallProgress, InstallStage, InstallationState, Manifest, VerificationOutcome,
};

use installer_gui::i18n::{t, Locale};

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
    window: Option<gtk::Window>,
    page: Page,
    verification: Option<VerificationOutcome>,
    install_state: Option<InstallationState>,
    staged_path: Option<PathBuf>,
    latest_progress: Option<InstallProgress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Manifest,
    Installing,
    Done,
    Error(String),
}

pub fn run() {
    let application = adw::Application::builder()
        .application_id("io.openinstall.installer")
        .build();

    application.connect_activate(|app| {
        let locale = Locale::detect();
        let manifest_path = std::env::args().nth(1).map(PathBuf::from);
        let environment = Environment::detect();
        let installer = Installer::default();

        let (manifest, _load_error) = match manifest_path.as_ref() {
            Some(path) => match Manifest::from_path(path) {
                Ok(m) => (Some(m), None),
                Err(e) => (None, Some(e.to_string())),
            },
            None => (Some(demo_manifest()), None),
        };

        let install_state =
            manifest.as_ref().and_then(|m| installer.inspect(m, &environment).ok());

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
        }));

        build_window(app, data);
    });

    application.run();
}

fn build_window(app: &adw::Application, data: Rc<RefCell<UiData>>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(700)
        .build();

    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new(
            t(data.borrow().locale, "app.title"),
            t(data.borrow().locale, "app.subtitle"),
        ))
        .build();

    let nav = adw::NavigationView::new();
    let toolbar = adw::ToolbarView::new();
    toolbar.set_content(Some(&nav));
    toolbar.add_top_bar(&header);
    window.set_content(Some(&toolbar));

    let (tx, rx) = mpsc::channel::<ProgressUpdate>();

    let page_content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page_content.set_hexpand(true);
    page_content.set_vexpand(true);

    data.borrow_mut().window = Some(window.upcast_ref::<gtk::Window>().clone());

    let nav_page = adw::NavigationPage::builder()
        .title(t(data.borrow().locale, "app.title"))
        .child(&page_content)
        .build();
    nav.push(&nav_page);

    let refresh: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

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
                    Ok(outcome) => d.verification = Some(outcome),
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

fn render_manifest(data: &Rc<RefCell<UiData>>, parent: &gtk::Box, tx: &mpsc::Sender<ProgressUpdate>) {
    let current = data.borrow();
    let locale = current.locale;
    let manifest = match &current.manifest {
        Some(m) => m.clone(),
        None => return,
    };
    let env = current.environment.clone();
    let install_state = current.install_state.clone();
    let verification = current.verification.clone();
    drop(current);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    scroll.set_child(Some(&content));
    parent.append(&scroll);

    if verification.as_ref().is_some_and(|v| v.signature_ok == Some(false)) {
        let warn = gtk::Label::builder()
            .label(t(locale, "manifest.signature_warning"))
            .xalign(0.0)
            .wrap(true)
            .css_classes(["error"])
            .margin_bottom(12)
            .build();
        content.append(&warn);
    }

    let name_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    if let Some(ref img) = manifest.image {
        let pic = gtk::Picture::for_filename(img);
        pic.set_width_request(64);
        pic.set_height_request(64);
        name_row.append(&pic);
    }

    let name_label = gtk::Label::builder()
        .label(&manifest.name)
        .xalign(0.0)
        .css_classes(["title-1"])
        .build();
    name_row.append(&name_label);
    content.append(&name_row);

    let pub_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let pub_label = gtk::Label::builder()
        .label(&format!("{}: {}", t(locale, "detail.publisher"), manifest.publisher))
        .xalign(0.0)
        .build();
    let ver_label = gtk::Label::builder()
        .label(&format!("{}: {}", t(locale, "detail.version"), manifest.version))
        .xalign(0.0)
        .build();
    pub_row.append(&pub_label);
    pub_row.append(&ver_label);
    content.append(&pub_row);

    let desc_label = gtk::Label::builder()
        .label(&manifest.description)
        .xalign(0.0)
        .wrap(true)
        .selectable(true)
        .build();
    content.append(&desc_label);

    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .tightening_threshold(400)
        .build();
    let group = adw::PreferencesGroup::new();
    clamp.set_child(Some(&group));

    add_detail_row(&group, t(locale, "detail.publisher"), &manifest.publisher);
    add_detail_row(&group, t(locale, "detail.version"), &manifest.version);
    add_detail_row(&group, t(locale, "detail.license"), manifest.license.as_deref().unwrap_or(t(locale, "detail.not_set")));
    add_detail_row(&group, t(locale, "detail.homepage"), manifest.homepage.as_deref().unwrap_or(t(locale, "detail.not_set")));
    add_detail_row(&group, t(locale, "detail.architecture"), &env.architecture);
    add_detail_row(&group, t(locale, "detail.distribution"), &env.distro);
    add_detail_row(&group, t(locale, "detail.package_manager"), &env.package_manager.to_string());
    if let Some(changelog) = &manifest.changelog {
        add_detail_row(&group, t(locale, "detail.changelog"), changelog);
    }

    let integrity = manifest.sha256.as_deref().map(|_| t(locale, "detail.available")).unwrap_or(t(locale, "detail.not_set"));
    add_detail_row(&group, t(locale, "detail.integrity"), integrity);

    let sig_status = match &verification {
        Some(v) => {
            if v.signature_ok == Some(true) { t(locale, "detail.valid") }
            else if v.signature_ok == Some(false) { t(locale, "detail.invalid") }
            else { t(locale, "detail.not_provided") }
        }
        None => {
            if manifest.signature.is_some() { t(locale, "detail.available") }
            else { t(locale, "detail.not_provided") }
        }
    };
    add_detail_row(&group, t(locale, "detail.signature"), sig_status);

    let selection = manifest.package_for_environment(&env);
    if let Some(ref pkg) = selection {
        add_detail_row(&group, t(locale, "detail.package"), pkg.reference);
    }
    content.append(&clamp);

    let state_label = gtk::Label::new(None);
    state_label.set_xalign(0.0);
    state_label.set_wrap(true);
    state_label.set_selectable(true);
    match &install_state {
        Some(InstallationState::NotInstalled) => {
            state_label.set_label(t(locale, "state.not_installed"));
        }
        Some(InstallationState::SameVersion { version }) => {
            state_label.set_label(&format!("{} ({})", t(locale, "state.same_version"), version));
        }
        Some(InstallationState::DifferentVersion { current_version, available_version }) => {
            state_label.set_label(&format!(
                "{}: {} \u{2192} {}",
                t(locale, "state.update_available"),
                current_version,
                available_version,
            ));
        }
        None => {}
    }
    content.append(&state_label);

    if let Some(ref v) = verification {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let sha_label = gtk::Label::builder()
            .xalign(0.0)
            .label(if v.sha256_ok { t(locale, "verify.sha256_ok") } else { t(locale, "verify.sha256_mismatch") })
            .build();
        vbox.append(&sha_label);
        let sig_label_text = match v.signature_ok {
            Some(true) => t(locale, "verify.signature_ok"),
            Some(false) => t(locale, "verify.signature_invalid"),
            None => t(locale, "verify.signature_missing"),
        };
        let sig_label = gtk::Label::builder()
            .xalign(0.0)
            .label(sig_label_text)
            .build();
        vbox.append(&sig_label);
        content.append(&vbox);
    }

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    button_box.set_margin_top(16);

    let install_btn = gtk::Button::builder()
        .label(match &install_state {
            Some(InstallationState::NotInstalled) | None => t(locale, "manifest.install"),
            Some(InstallationState::SameVersion { .. }) => t(locale, "manifest.reinstall"),
            Some(InstallationState::DifferentVersion { .. }) => t(locale, "manifest.update"),
        })
        .css_classes(["suggested-action"])
        .build();

    let remove_btn = gtk::Button::builder()
        .label(t(locale, "manifest.remove"))
        .css_classes(["destructive-action"])
        .build();

    let verify_btn = gtk::Button::with_label(t(locale, "manifest.verify"));
    let rollback_btn = gtk::Button::with_label(t(locale, "manifest.rollback"));
    let reload_btn = gtk::Button::with_label(t(locale, "manifest.reload"));
    let cache_btn = gtk::Button::with_label(t(locale, "manifest.cache_info"));
    let history_btn = gtk::Button::with_label(t(locale, "manifest.history"));

    button_box.append(&install_btn);
    button_box.append(&remove_btn);
    button_box.append(&verify_btn);
    button_box.append(&rollback_btn);
    button_box.append(&reload_btn);
    button_box.append(&cache_btn);
    button_box.append(&history_btn);
    content.append(&button_box);

    let has_pkg = selection.is_some();
    let bad_sig = verification.as_ref().map_or(false, |v| v.signature_ok == Some(false));
    install_btn.set_sensitive(has_pkg && !bad_sig);
    remove_btn.set_sensitive(has_pkg);
    verify_btn.set_sensitive(has_pkg);

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
            // Forward progress updates to GUI channel
            let tx_fwd = tx.clone();
            std::thread::spawn(move || {
                while let Ok(p) = progress_rx.recv() {
                    if tx_fwd.send(ProgressUpdate::Progress(p)).is_err() {
                        break;
                    }
                }
            });
            let result = installer.install_with_progress(&m, &env, progress_tx)
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
            let result = installer.remove(&m, &env)
                .map(|o| (o.command, o.staged_path))
                .map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::RemoveResult(result));
        });
    });

    let data_verify = Rc::clone(data);
    let tx_verify = tx.clone();
    verify_btn.connect_clicked(move |_| {
        let manifest = data_verify.borrow().manifest.clone();
        let env = data_verify.borrow().environment.clone();
        let installer = data_verify.borrow().installer.clone();
        let tx = tx_verify.clone();
        std::thread::spawn(move || {
            let m = match manifest {
                Some(ref m) => m.clone(),
                None => return,
            };
            let result = installer.verify(&m, &env).map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::VerifyResult(result));
        });
    });

    let data_rollback = Rc::clone(data);
    let tx_rollback = tx.clone();
    rollback_btn.connect_clicked(move |_| {
        let mut d = data_rollback.borrow_mut();
        d.page = Page::Installing;
        let manifest = d.manifest.clone();
        let env = d.environment.clone();
        let installer = d.installer.clone();
        let tx = tx_rollback.clone();
        drop(d);
        std::thread::spawn(move || {
            let m = match manifest {
                Some(ref m) => m.clone(),
                None => return,
            };
            let result = installer.rollback(&m, &env)
                .map(|o| (o.command, o.staged_path))
                .map_err(|e| e.to_string());
            let _ = tx.send(ProgressUpdate::RollbackResult(result));
        });
    });

    let data_reload = Rc::clone(data);
    let parent_clone = parent.clone();
    let tx_reload = tx.clone();
    reload_btn.connect_clicked(move |_| {
        let mut d = data_reload.borrow_mut();
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
        let install_state = manifest.as_ref().and_then(|m| installer.inspect(m, &env).ok());
        d.environment = env;
        d.installer = installer;
        d.manifest = manifest;
        d.install_state = install_state;
        d.verification = None;
        d.staged_path = None;
        drop(d);
        clear_children(&parent_clone);
        render_manifest(&data_reload, &parent_clone, &tx_reload);
    });

    let data_cache = Rc::clone(data);
    cache_btn.connect_clicked(move |_| {
        let installer = data_cache.borrow().installer.clone();
        let result = installer.cache_info().map(|info| {
            format!("{} files, {} bytes", info.file_count, info.total_bytes)
        }).unwrap_or_else(|e| e.to_string());
        // Show in a transient dialog using gtk::AlertDialog (GTK4)
        let dialog = gtk::AlertDialog::builder()
            .message(t(data_cache.borrow().locale, "manifest.cache_info"))
            .detail(&result)
            .build();
        dialog.show(None::<&gtk::Window>);
    });

    let data_history = Rc::clone(data);
    history_btn.connect_clicked(move |_| {
        let installer = data_history.borrow().installer.clone();
        let result = installer.get_history().map(|entries| {
            if entries.is_empty() {
                "No history".to_string()
            } else {
                entries.iter().map(|e| {
                    format!("{} v{} via {} — {}",
                        e.package_id, e.version, e.package_manager, e.installed_at_unix_secs)
                }).collect::<Vec<_>>().join("\n")
            }
        }).unwrap_or_else(|e| e.to_string());
        let dialog = gtk::AlertDialog::builder()
            .message(t(data_history.borrow().locale, "manifest.history"))
            .detail(&result)
            .build();
        dialog.show(None::<&gtk::Window>);
    });
}

fn render_installing(data: &Rc<RefCell<UiData>>, parent: &gtk::Box) {
    let locale = data.borrow().locale;

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);
    vbox.set_margin_top(48);

    let label = gtk::Label::builder()
        .label(t(locale, "install.progress"))
        .css_classes(["title-2"])
        .build();
    vbox.append(&label);

    let progress = gtk::ProgressBar::new();
    progress.set_show_text(true);
    let stored = data.borrow().latest_progress.clone();
    if let Some(p) = stored {
        if p.total_bytes > 0 {
            let frac = p.downloaded_bytes as f64 / p.total_bytes as f64;
            progress.set_fraction(frac.clamp(0.0, 1.0));
            if p.stage == InstallStage::Done {
                progress.set_fraction(1.0);
            }
        }
    } else {
        progress.set_pulse_step(0.05);
        progress.pulse();
    }
    progress.set_margin_bottom(12);
    vbox.append(&progress);

    let log_buf = gtk::TextBuffer::new(None);
    let log_view = gtk::TextView::builder()
        .buffer(&log_buf)
        .editable(false)
        .wrap_mode(gtk::WrapMode::Word)
        .height_request(200)
        .width_request(500)
        .build();
    let log_scroll = gtk::ScrolledWindow::builder()
        .child(&log_view)
        .vexpand(true)
        .hexpand(true)
        .build();
    vbox.append(&log_scroll);

    parent.append(&vbox);
}

fn render_done(data: &Rc<RefCell<UiData>>, parent: &gtk::Box) {
    let locale = data.borrow().locale;
    let app_name = data.borrow().manifest.as_ref().map(|m| m.name.clone());
    let staged = data.borrow().staged_path.clone();
    let _ = data;

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);
    vbox.set_margin_top(48);

    let icon = gtk::Image::from_icon_name("object-select-symbolic");
    icon.set_pixel_size(64);
    vbox.append(&icon);

    let title = gtk::Label::builder()
        .label(t(locale, "done.title"))
        .css_classes(["title-1"])
        .build();
    vbox.append(&title);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let launch_btn = gtk::Button::builder()
        .label(t(locale, "done.launch"))
        .css_classes(["suggested-action"])
        .build();
    let name = app_name.clone().unwrap_or_default();
    launch_btn.connect_clicked(move |_| {
        let safe_name = name.replace('/', "_");
        let _ = SysCommand::new(&safe_name).spawn();
    });
    btn_box.append(&launch_btn);

    let open_btn = gtk::Button::builder()
        .label(t(locale, "done.open_folder"))
        .build();
    let staged_clone = staged.clone();
    open_btn.connect_clicked(move |_| {
        if let Some(ref path) = staged_clone {
            if let Some(parent) = path.parent() {
                let _ = SysCommand::new("xdg-open")
                    .arg(parent.to_str().unwrap_or("."))
                    .spawn();
            }
        }
    });
    btn_box.append(&open_btn);

    let close_btn = gtk::Button::builder()
        .label(t(locale, "done.close"))
        .build();
    let data_close = Rc::clone(data);
    close_btn.connect_clicked(move |_| {
        if let Some(ref window) = data_close.borrow().window {
            window.close();
        }
    });
    btn_box.append(&close_btn);

    vbox.append(&btn_box);
    parent.append(&vbox);
}

fn render_error(data: &Rc<RefCell<UiData>>, parent: &gtk::Box, message: String) {
    let locale = data.borrow().locale;

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);
    vbox.set_margin_top(48);

    let icon = gtk::Image::from_icon_name("dialog-error-symbolic");
    icon.set_pixel_size(64);
    vbox.append(&icon);

    let title = gtk::Label::builder()
        .label(t(locale, "error.title"))
        .css_classes(["title-1"])
        .build();
    vbox.append(&title);

    let msg = gtk::Label::builder()
        .label(&message)
        .wrap(true)
        .selectable(true)
        .max_width_chars(60)
        .build();
    vbox.append(&msg);

    let close_btn = gtk::Button::builder()
        .label(t(locale, "error.close"))
        .css_classes(["suggested-action"])
        .build();
    let data_close = Rc::clone(data);
    close_btn.connect_clicked(move |_| {
        if let Some(ref window) = data_close.borrow().window {
            window.close();
        }
    });
    vbox.append(&close_btn);

    parent.append(&vbox);
}

fn add_detail_row(group: &adw::PreferencesGroup, title: &str, subtitle: &str) {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    group.add(&row);
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
            fallback: Some("https://example.com/demo.AppImage".to_string()),
        },
        sha256: None,
        signature: None,
    }
}
