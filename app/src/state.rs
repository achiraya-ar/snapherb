use chrono::{Datelike, Days, Local, NaiveDate, NaiveDateTime};
use leptos::prelude::*;
use snap_herb_models::{
    AppSettings, ApprovalSettings, ConnectionSummary, ReportMeta, ReportRow, RequisitionPeriod,
    RequisitionRecord, SessionUser,
};

pub fn latest_selectable_date() -> NaiveDate {
    let today = Local::now().date_naive();
    today.checked_sub_days(Days::new(1)).unwrap_or(today)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ConnectionHealth {
    Unconfigured,
    Connected,
    Disconnected,
}

impl ConnectionHealth {
    pub fn label_th(self) -> &'static str {
        match self {
            Self::Unconfigured => "ยังไม่ตั้งค่าการเชื่อมต่อ",
            Self::Connected => "เชื่อมต่อ HOSxP แล้ว",
            Self::Disconnected => "การเชื่อมต่อขัดข้อง",
        }
    }

    pub fn dot_class(self) -> &'static str {
        match self {
            Self::Unconfigured => "idle",
            Self::Connected => "ok",
            Self::Disconnected => "down",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Connection,
    Doctors,
    Catalog,
    Approvals,
    Report,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Banner {
    pub kind: BannerKind,
    pub message: String,
}

impl Banner {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: BannerKind::Error,
            message: message.into(),
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self {
            kind: BannerKind::Warn,
            message: message.into(),
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: BannerKind::Info,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedRequisition {
    pub rows: Vec<ReportRow>,
    pub period: RequisitionPeriod,
    pub report: ReportMeta,
    pub approvals: ApprovalSettings,
    pub confirmed_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEdit {
    pub icode: String,
    pub name: String,
    pub strength_text: String,
    pub usage_unit: String,
    pub pack_unit: String,
    pub max_packs: f64,
    pub usage: f64,
    pub old_requested: f64,
    pub new_requested: f64,
}

#[derive(Clone)]
pub struct AppState {
    pub settings: RwSignal<Option<AppSettings>>,
    pub configured: RwSignal<bool>,
    pub connection_summary: RwSignal<Option<ConnectionSummary>>,
    pub connection_error: RwSignal<Option<String>>,
    pub session: RwSignal<Option<SessionUser>>,
    pub auth_checked: RwSignal<bool>,
    pub health: RwSignal<ConnectionHealth>,
    pub settings_open: RwSignal<bool>,
    pub settings_tab: RwSignal<SettingsTab>,
    pub start_date: RwSignal<NaiveDate>,
    pub end_date: RwSignal<NaiveDate>,
    pub rows: RwSignal<Option<Vec<ReportRow>>>,
    pub confirmed: RwSignal<Option<ConfirmedRequisition>>,
    pub current_record: RwSignal<Option<RequisitionRecord>>,
    pub pending_edit: RwSignal<Option<PendingEdit>>,
    pub add_drug_open: RwSignal<bool>,
    pub confirm_open: RwSignal<bool>,
    pub history: RwSignal<Vec<RequisitionRecord>>,
    pub history_open: RwSignal<bool>,
    pub history_loading: RwSignal<bool>,
    pub reprint: RwSignal<Option<RequisitionRecord>>,
    pub loading: RwSignal<bool>,
    pub banner: RwSignal<Option<Banner>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let latest = latest_selectable_date();
        let first = NaiveDate::from_ymd_opt(latest.year(), latest.month(), 1).unwrap_or(latest);
        Self {
            settings: RwSignal::new(None),
            configured: RwSignal::new(false),
            connection_summary: RwSignal::new(None),
            connection_error: RwSignal::new(None),
            session: RwSignal::new(None),
            auth_checked: RwSignal::new(false),
            health: RwSignal::new(ConnectionHealth::Unconfigured),
            settings_open: RwSignal::new(false),
            settings_tab: RwSignal::new(SettingsTab::Connection),
            start_date: RwSignal::new(first),
            end_date: RwSignal::new(latest),
            rows: RwSignal::new(None),
            confirmed: RwSignal::new(None),
            current_record: RwSignal::new(None),
            pending_edit: RwSignal::new(None),
            add_drug_open: RwSignal::new(false),
            confirm_open: RwSignal::new(false),
            history: RwSignal::new(Vec::new()),
            history_open: RwSignal::new(false),
            history_loading: RwSignal::new(false),
            reprint: RwSignal::new(None),
            loading: RwSignal::new(false),
            banner: RwSignal::new(None),
        }
    }

    pub fn period(&self) -> Option<RequisitionPeriod> {
        RequisitionPeriod::new(self.start_date.get(), self.end_date.get()).ok()
    }

    pub fn period_label(&self) -> String {
        self.period()
            .map(|period| period.thai_range())
            .unwrap_or_else(|| "ช่วงวันที่ไม่ถูกต้อง".to_string())
    }

    pub fn finish_login(&self, user: SessionUser) {
        self.session.set(Some(user));
        self.clear_requisition();
        self.banner.set(None);
    }

    pub fn clear_session(&self) {
        self.session.set(None);
        self.clear_requisition();
        self.settings_open.set(false);
        self.history_open.set(false);
        self.reprint.set(None);
        self.banner.set(None);
    }

    pub fn clear_requisition(&self) {
        self.rows.set(None);
        self.confirmed.set(None);
        self.current_record.set(None);
        self.pending_edit.set(None);
        self.add_drug_open.set(false);
        self.confirm_open.set(false);
        self.reprint.set(None);
    }

    pub fn invalidate_confirmation(&self) {
        if self.current_record.get_untracked().is_some() {
            return;
        }
        self.confirmed.set(None);
        self.confirm_open.set(false);
    }

    pub fn confirm_requisition(&self) -> bool {
        let Some(rows) = self.rows.get_untracked() else {
            return false;
        };
        let Some(period) = self.period() else {
            return false;
        };
        let Some(settings) = self.settings.get_untracked() else {
            return false;
        };
        if rows.is_empty() {
            return false;
        }
        self.confirmed.set(Some(ConfirmedRequisition {
            rows,
            period,
            report: settings.report,
            approvals: settings.approvals,
            confirmed_at: Local::now().naive_local(),
        }));
        self.set_info_banner("ยืนยันข้อมูลเบิกแล้ว พร้อมพิมพ์ใบเบิกทั้ง 2 ฟอร์ม");
        true
    }

    pub fn print_snapshot_matches(&self) -> bool {
        let Some(confirmed) = self.confirmed.get_untracked() else {
            return false;
        };
        let Some(rows) = self.rows.get_untracked() else {
            return false;
        };
        let Some(period) = self.period() else {
            return false;
        };
        let Some(settings) = self.settings.get_untracked() else {
            return false;
        };
        rows == confirmed.rows
            && period == confirmed.period
            && settings.report == confirmed.report
            && settings.approvals == confirmed.approvals
    }

    pub fn set_info_banner(&self, message: impl Into<String>) {
        self.banner.set(Some(Banner::info(message)));
    }

    pub fn catalog_is_empty(&self) -> bool {
        self.settings
            .get()
            .is_some_and(|settings| settings.catalog.is_empty())
    }

    pub fn doctors_are_empty(&self) -> bool {
        self.settings
            .get()
            .is_some_and(|settings| settings.doctors.is_empty())
    }

    pub fn fetch_usage_action(&self) {
        let Some(period) = self.period() else {
            self.banner
                .set(Some(Banner::error("ช่วงวันที่ไม่ถูกต้อง: วันที่เริ่มต้องไม่หลังวันที่สิ้นสุด")));
            return;
        };
        if period.end_date > latest_selectable_date() {
            self.banner
                .set(Some(Banner::warn("เลือกช่วงวันที่ได้ถึงวันที่ก่อนหน้าวันที่ปัจจุบันเท่านั้น")));
            return;
        }
        let Some(settings) = self.settings.get() else {
            self.banner.set(Some(Banner::error("ยังโหลดการตั้งค่าไม่เสร็จ")));
            return;
        };
        if settings.catalog.is_empty() {
            self.settings_tab.set(SettingsTab::Catalog);
            self.settings_open.set(true);
            self.banner
                .set(Some(Banner::warn("ยังไม่มีรายการยาสมุนไพรในบัญชี")));
            return;
        }
        if settings.doctors.is_empty() {
            self.settings_tab.set(SettingsTab::Doctors);
            self.settings_open.set(true);
            self.banner
                .set(Some(Banner::warn("ยังไม่ได้เลือกแพทย์ที่ต้องการตรวจจับการสั่งจ่ายยา")));
            return;
        }

        let state = self.clone();
        state.loading.set(true);
        state.banner.set(None);
        leptos::task::spawn_local(async move {
            match crate::api::period_context(&period).await {
                Ok(context) => {
                    if !context.conflicts.is_empty() {
                        let documents = context
                            .conflicts
                            .iter()
                            .map(|record| record.document_no.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        state.banner.set(Some(Banner::warn(format!(
                            "ช่วงวันที่นี้มีใบเบิกแล้ว (เลขที่ {documents}) จึงไม่สามารถแก้ไขหรือเบิกซ้ำได้ กรุณาเปิดประวัติการเบิกเพื่อพิมพ์ซ้ำ"
                        ))));
                        state.loading.set(false);
                        return;
                    }
                    match crate::api::fetch_usage(&period).await {
                        Ok(usage) => {
                            let rows = snap_herb_core::build_report(&settings.catalog, &usage);
                            state.rows.set(Some(rows));
                            state.confirmed.set(None);
                            state.current_record.set(None);
                        }
                        Err(err) => set_api_error(&state, err),
                    }
                }
                Err(err) => set_api_error(&state, err),
            }
            state.loading.set(false);
        });
    }

    pub fn load_history_action(&self) {
        let state = self.clone();
        state.history_loading.set(true);
        leptos::task::spawn_local(async move {
            match crate::api::list_requisitions().await {
                Ok(records) => state.history.set(records),
                Err(err) => set_api_error(&state, err),
            }
            state.history_loading.set(false);
        });
    }

    pub fn open_history_action(&self) {
        self.history_open.set(true);
        self.load_history_action();
    }
}

fn set_api_error(state: &AppState, err: crate::api::ApiError) {
    if err.kind == crate::api::ApiErrorKind::NotConfigured {
        state.settings_tab.set(SettingsTab::Connection);
    }
    let banner = if matches!(
        err.kind,
        crate::api::ApiErrorKind::NotConfigured
            | crate::api::ApiErrorKind::Locked
            | crate::api::ApiErrorKind::Validation
    ) {
        Banner::warn(err.message)
    } else {
        Banner::error(err.message)
    };
    state.banner.set(Some(banner));
}

#[cfg(test)]
mod tests {
    use super::*;
    use snap_herb_models::ReportRow;

    fn row() -> ReportRow {
        ReportRow {
            icode: "1001".into(),
            name: "ขมิ้นชัน".into(),
            strength_text: "500 mg".into(),
            usage_unit: "เม็ด".into(),
            pack_unit: "กล่อง".into(),
            units_per_pack: 100.0,
            max_packs: 30.0,
            usage: 130.0,
            requested: 1.0,
            added: false,
            reason: String::new(),
        }
    }

    #[test]
    fn default_dates_end_before_today() {
        let state = AppState::new();
        assert!(state.start_date.get_untracked() <= state.end_date.get_untracked());
        assert!(state.end_date.get_untracked() <= latest_selectable_date());
        assert!(state.period().is_some());
    }

    #[test]
    fn confirmation_requires_rows_and_is_invalidated_by_edits() {
        let state = AppState::new();
        state.settings.set(Some(AppSettings::default()));
        state
            .start_date
            .set(NaiveDate::from_ymd_opt(2026, 9, 1).expect("date"));
        state
            .end_date
            .set(NaiveDate::from_ymd_opt(2026, 9, 30).expect("date"));
        state.rows.set(Some(vec![row()]));
        assert!(state.confirm_requisition());
        assert!(state.print_snapshot_matches());

        let mut rows = state.rows.get_untracked().expect("rows");
        rows[0].requested = 2.0;
        state.rows.set(Some(rows));
        assert!(!state.print_snapshot_matches());
        state.invalidate_confirmation();
        assert!(state.confirmed.get_untracked().is_none());
    }

    #[test]
    fn health_labels_are_stable() {
        assert_eq!(ConnectionHealth::Connected.label_th(), "เชื่อมต่อ HOSxP แล้ว");
        assert_eq!(ConnectionHealth::Disconnected.dot_class(), "down");
    }

    #[test]
    fn period_requires_ordered_dates() {
        let state = AppState::new();
        state
            .start_date
            .set(NaiveDate::from_ymd_opt(2026, 10, 31).expect("date"));
        state
            .end_date
            .set(NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"));
        assert!(state.period().is_none());
    }
}
