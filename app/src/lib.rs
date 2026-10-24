pub mod api;
pub mod components;
pub mod settings_io;
pub mod state;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use snap_herb_models::{AppSettings, RequisitionRecord};

use components::command_bar::CommandBar;
use components::document_sheet::{DocumentSheet, SheetBanner, SheetData};
use components::empty_state::{CatalogEmpty, DoctorsEmpty, NoDataHint};
use components::history_modal::{HistoryModal, ReprintBar};
use components::login_screen::{LoginScreen, StartupSplash};
use components::official_sheet::OfficialSheet;
use components::requisition_dialogs::RequisitionDialogs;
use components::settings_modal::SettingsModal;
use components::top_bar::TopBar;
use state::{AppState, Banner};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(30);

type PollLoop = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

pub fn run() {
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();

    let load_state = state.clone();
    leptos::task::spawn_local(async move {
        load_initial_state(load_state).await;
    });

    let poll_state = state.clone();
    let poll_health = Rc::new(move || {
        let poll_state = poll_state.clone();
        leptos::task::spawn_local(async move {
            if let Ok(health) = api::connection_health().await {
                poll_state.health.set(health);
            }
        });
    });
    let poll_loop: PollLoop = Rc::new(RefCell::new(None));
    let trigger = {
        let poll_loop = Rc::clone(&poll_loop);
        Rc::new(move || {
            poll_health();
            if let Some(next) = poll_loop.borrow().as_ref().map(Rc::clone) {
                let _ = set_timeout_with_handle(move || next(), HEALTH_POLL_INTERVAL);
            }
        })
    };
    *poll_loop.borrow_mut() = Some(Rc::clone(&trigger) as Rc<dyn Fn()>);
    trigger();

    let shell_state = state.clone();
    view! {
        <div class="app">
            {move || {
                let state = shell_state.clone();
                if !state.auth_checked.get() {
                    view! { <StartupSplash /> }.into_any()
                } else if state.session.get().is_none() {
                    view! { <LoginScreen state=state.clone() /> }.into_any()
                } else if let Some(record) = state.reprint.get() {
                    let data = sheet_from_record(&record);
                    view! {
                        <TopBar state=state.clone() />
                        <ReprintBar state=state.clone() record=record />
                        <main class="stage">
                            <SheetBanner state=state.clone() />
                            <p class="document-label">"ใบเบิกจากระบบ (แบบเดิม)"</p>
                            <DocumentSheet state=state.clone() data=data.clone() />
                            <p class="document-label">"ใบเบิกพัสดุ (แบบทางการ)"</p>
                            <OfficialSheet data=data />
                        </main>
                    }
                        .into_any()
                } else {
                    let data = sheet_from_state(&state);
                    view! {
                        <TopBar state=state.clone() />
                        <CommandBar state=state.clone() />
                        <main class="stage">
                            <SheetBanner state=state.clone() />
                            {if state.catalog_is_empty() {
                                view! { <CatalogEmpty state=state.clone() /> }.into_any()
                            } else if state.doctors_are_empty() {
                                view! { <DoctorsEmpty state=state.clone() /> }.into_any()
                            } else {
                                match data {
                                    Some(data) => {
                                        view! {
                                            <p class="document-label">"ใบเบิกจากระบบ (แบบเดิม)"</p>
                                            <DocumentSheet state=state.clone() data=data.clone() />
                                            <p class="document-label">"ใบเบิกพัสดุ (แบบทางการ)"</p>
                                            <OfficialSheet data=data />
                                        }
                                            .into_any()
                                    }
                                    None => {
                                        view! { <NoDataHint state=state.clone() /> }.into_any()
                                    }
                                }
                            }}
                        </main>
                    }
                        .into_any()
                }
            }}
            <SettingsModal state=state.clone() />
            <HistoryModal state=state.clone() />
            <RequisitionDialogs state=state.clone() />
        </div>
    }
}

fn sheet_from_state(state: &AppState) -> Option<SheetData> {
    if let Some(record) = state.current_record.get() {
        return Some(sheet_from_record(&record));
    }
    let rows = state.rows.get()?;
    let settings = state.settings.get()?;
    let requested_by = state
        .session
        .get()
        .map(|user| user.display_name.trim().to_string())
        .filter(|name| !name.is_empty());
    Some(SheetData {
        rows,
        period: state.period(),
        report: settings.report,
        approvals: settings.approvals,
        document_no: None,
        printed_at: None,
        requested_by,
        editable: true,
    })
}

fn sheet_from_record(record: &RequisitionRecord) -> SheetData {
    SheetData {
        rows: record.rows.clone(),
        period: Some(record.period),
        report: record.report.clone(),
        approvals: record.approvals.clone(),
        document_no: Some(record.document_no.clone()),
        printed_at: Some(record.printed_at),
        requested_by: Some(record.requested_by.clone()),
        editable: false,
    }
}

async fn load_initial_state(state: AppState) {
    let configured = api::connection_status().await;
    state.configured.set(configured);

    match api::load_settings().await {
        Ok(settings) => state.settings.set(Some(settings)),
        Err(err) => {
            state.settings.set(Some(AppSettings::default()));
            state.banner.set(Some(Banner::error(err.message)));
        }
    }

    if configured {
        match api::connection_summary().await {
            Ok(summary) => state.connection_summary.set(summary),
            Err(err) => state.connection_error.set(Some(err.message)),
        }
        match api::reconnect().await {
            Ok(_) => {
                state.health.set(state::ConnectionHealth::Connected);
                state.connection_error.set(None);
            }
            Err(err) => {
                state.health.set(state::ConnectionHealth::Disconnected);
                state.connection_error.set(Some(err.message));
            }
        }
    } else {
        state.connection_summary.set(None);
        state.connection_error.set(None);
        state.health.set(state::ConnectionHealth::Unconfigured);
    }

    if let Ok(Some(user)) = api::current_session().await {
        state.finish_login(user);
    }
    state.auth_checked.set(true);
}
