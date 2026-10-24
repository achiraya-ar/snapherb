use std::collections::HashSet;
use std::time::{Duration, Instant};

use chrono::Local;
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Serialize};
use snap_herb_core::{HosxRepository, RepositoryError, existing_conflicts, verify_password};
use snap_herb_hosxp_connector::config::{HosxConfig, load_vault};
use snap_herb_hosxp_connector::{ConnectorRepository, MySqlPool, connect as connect_hosxp};
use snap_herb_models::{
    AppSettings, ApprovalSettings, ConnectionSummary, DoctorRef, DrugItem, HerbCatalog,
    PeriodContext, ReportMeta, ReportRow, RequisitionPeriod, RequisitionRecord, SessionUser,
    UsageRow,
};
use tauri::State;

use crate::error::CommandError;
use crate::requisitions::{self, build_record};
use crate::settings::{self, AppSettings as Settings};
use crate::state::{AppState, ConnectionHealth};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(30);
const SEARCH_TERM_MIN_CHARS: usize = 3;
const SEARCH_TERM_MAX_CHARS: usize = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl ConnectionInput {
    fn validate(&self) -> Result<(), CommandError> {
        if self.host.trim().is_empty() {
            return Err(CommandError::validation("กรุณากรอกที่อยู่เซิร์ฟเวอร์"));
        }
        if self.port == 0 {
            return Err(CommandError::validation("พอร์ตต้องอยู่ระหว่าง 1-65535"));
        }
        if self.database.trim().is_empty() {
            return Err(CommandError::validation("กรุณากรอกชื่อฐานข้อมูล"));
        }
        if self.user.trim().is_empty() {
            return Err(CommandError::validation("กรุณากรอกชื่อผู้ใช้"));
        }
        if self.password.is_empty() {
            return Err(CommandError::validation("กรุณากรอกรหัสผ่าน"));
        }
        Ok(())
    }

    fn into_config(self) -> HosxConfig {
        let ConnectionInput {
            host,
            port,
            database,
            user,
            password,
        } = self;
        HosxConfig::new(
            host.trim().to_string(),
            port,
            database.trim().to_string(),
            user.trim().to_string(),
            SecretBox::from(password),
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub latency_ms: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[tauri::command]
pub async fn connection_status(state: State<'_, AppState>) -> Result<bool, CommandError> {
    Ok(state.config_path().exists())
}

#[tauri::command]
pub async fn connection_health(
    state: State<'_, AppState>,
) -> Result<ConnectionHealth, CommandError> {
    Ok(*state.health.lock().await)
}

#[tauri::command]
pub async fn configure_connection(
    input: ConnectionInput,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    input.validate()?;
    let config = input.into_config();

    let vault = load_vault().map_err(CommandError::from)?;
    snap_herb_hosxp_connector::config::save_encrypted(&state.config_path(), &vault, &config)
        .await
        .map_err(CommandError::from)?;

    *state.pool.lock().await = None;
    *state.session.lock().await = None;
    let probe_state = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        let health = probe_health(&probe_state).await;
        *probe_state.health.lock().await = health;
    });
    Ok(())
}

#[tauri::command]
pub async fn connection_summary(
    state: State<'_, AppState>,
) -> Result<Option<ConnectionSummary>, CommandError> {
    if !state.config_path().exists() {
        return Ok(None);
    }
    let vault = load_vault().map_err(CommandError::from)?;
    let config = snap_herb_hosxp_connector::config::load(&state.config_path(), &vault)
        .await
        .map_err(CommandError::from)?
        .ok_or_else(CommandError::not_configured)?;
    Ok(Some(ConnectionSummary {
        host: config.host,
        port: config.port,
        database: config.database,
        user: config.user,
    }))
}

#[tauri::command]
pub async fn connect(
    input: ConnectionInput,
    state: State<'_, AppState>,
) -> Result<ConnectionTestResult, CommandError> {
    input.validate()?;
    let config = input.into_config();
    let started = Instant::now();
    let pool = match connect_hosxp(&config).await {
        Ok(pool) => pool,
        Err(err) => {
            *state.health.lock().await = ConnectionHealth::Disconnected;
            return Err(err.into());
        }
    };
    if let Err(err) = ConnectorRepository::new(pool.clone()).ping().await {
        *state.pool.lock().await = None;
        *state.health.lock().await = ConnectionHealth::Disconnected;
        return Err(err.into());
    }

    let vault = load_vault().map_err(CommandError::from)?;
    snap_herb_hosxp_connector::config::save_encrypted(&state.config_path(), &vault, &config)
        .await
        .map_err(CommandError::from)?;
    *state.pool.lock().await = Some(pool);
    *state.health.lock().await = ConnectionHealth::Connected;
    *state.session.lock().await = None;
    Ok(ConnectionTestResult {
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn reconnect(state: State<'_, AppState>) -> Result<ConnectionTestResult, CommandError> {
    let started = Instant::now();
    let repository = acquire_repository(&state).await?;
    if let Err(err) = repository.ping().await {
        *state.pool.lock().await = None;
        *state.health.lock().await = ConnectionHealth::Disconnected;
        return Err(err.into());
    }
    *state.health.lock().await = ConnectionHealth::Connected;
    Ok(ConnectionTestResult {
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn login(
    input: LoginInput,
    state: State<'_, AppState>,
) -> Result<SessionUser, CommandError> {
    let username = input.username.trim();
    if username.is_empty() {
        return Err(CommandError::validation("กรุณากรอกชื่อผู้ใช้"));
    }
    if username.chars().count() > 100 {
        return Err(CommandError::validation("ชื่อผู้ใช้ยาวเกินไป"));
    }
    let password = SecretString::from(input.password);
    if password.expose_secret().is_empty() {
        return Err(CommandError::validation("กรุณากรอกรหัสผ่าน"));
    }
    if password.expose_secret().chars().count() > 200 {
        return Err(CommandError::validation("รหัสผ่านยาวเกินไป"));
    }

    *state.session.lock().await = None;
    let repository = acquire_repository(&state).await?;
    let account = repository.fetch_officer(username).await?;
    let Some(account) = account else {
        return Err(CommandError::auth("ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง"));
    };
    if account.auth_type_id != 1 {
        return Err(CommandError::auth(
            "บัญชีนี้ใช้การยืนยันตัวตนแบบ LDAP ซึ่งยังไม่เปิดใช้งาน",
        ));
    }
    if !account.active.eq_ignore_ascii_case("Y") {
        return Err(CommandError::auth("บัญชีนี้ถูกปิดใช้งาน กรุณาติดต่อผู้ดูแลระบบ"));
    }
    if !verify_password(password.expose_secret(), &account.password_md5) {
        return Err(CommandError::auth("ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง"));
    }

    let display_name = if account.display_name.trim().is_empty() {
        account.login_name.clone()
    } else {
        account.display_name
    };
    let user = SessionUser {
        officer_id: account.officer_id,
        login_name: account.login_name,
        display_name,
        doctor_code: account.doctor_code,
        role: "STAFF".to_string(),
    };
    *state.session.lock().await = Some(user.clone());
    Ok(user)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), CommandError> {
    *state.session.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn current_session(
    state: State<'_, AppState>,
) -> Result<Option<SessionUser>, CommandError> {
    Ok(state.session.lock().await.clone())
}

#[tauri::command]
pub async fn test_connection(input: ConnectionInput) -> Result<ConnectionTestResult, CommandError> {
    input.validate()?;
    let config = input.into_config();
    let started = Instant::now();
    let pool = connect_hosxp(&config).await.map_err(CommandError::from)?;
    ConnectorRepository::new(pool)
        .ping()
        .await
        .map_err(CommandError::from)?;
    Ok(ConnectionTestResult {
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn save_doctors(
    doctors: Vec<DoctorRef>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    let doctors = normalize_doctors(doctors)?;
    update_settings(&state, |settings| settings.doctors = doctors).await
}

#[tauri::command]
pub async fn save_catalog(
    catalog: HerbCatalog,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    let mut catalog = catalog;
    for drug in &mut catalog.drugs {
        drug.icode = drug.icode.trim().to_string();
        drug.name = drug.name.trim().to_string();
        drug.strength_text = drug.strength_text.trim().to_string();
        drug.usage_unit = drug.usage_unit.trim().to_string();
        drug.pack_unit = drug.pack_unit.trim().to_string();
    }
    validate_catalog(&catalog)?;
    update_settings(&state, |settings| settings.catalog = catalog).await
}

#[tauri::command]
pub async fn save_report(
    report: ReportMeta,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    validate_report(&report)?;
    update_settings(&state, |settings| settings.report = report).await
}

#[tauri::command]
pub async fn save_approvals(
    approvals: ApprovalSettings,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    let approvals = normalize_approvals(approvals);
    update_settings(&state, |settings| settings.approvals = approvals).await
}

#[tauri::command]
pub async fn search_drugs(
    term: String,
    state: State<'_, AppState>,
) -> Result<Vec<DrugItem>, CommandError> {
    require_session(&state).await?;
    let term = term.trim();
    let char_count = term.chars().count();
    if char_count < SEARCH_TERM_MIN_CHARS {
        return Err(CommandError::validation("พิมพ์อย่างน้อย 3 ตัวอักษรเพื่อค้นหายา"));
    }
    if char_count > SEARCH_TERM_MAX_CHARS {
        return Err(CommandError::validation("คำค้นหายาวเกินไป"));
    }

    let repository = acquire_repository(&state).await?;
    match repository.search_drugs(term).await {
        Ok(hits) => Ok(hits),
        Err(err) => {
            note_query_outcome(&state, &err).await;
            Err(err.into())
        }
    }
}

#[tauri::command]
pub async fn search_doctors(
    term: String,
    state: State<'_, AppState>,
) -> Result<Vec<DoctorRef>, CommandError> {
    require_session(&state).await?;
    let term = term.trim();
    let char_count = term.chars().count();
    if char_count < SEARCH_TERM_MIN_CHARS {
        return Err(CommandError::validation("พิมพ์อย่างน้อย 3 ตัวอักษรเพื่อค้นหาแพทย์"));
    }
    if char_count > SEARCH_TERM_MAX_CHARS {
        return Err(CommandError::validation("คำค้นหายาวเกินไป"));
    }

    let repository = acquire_repository(&state).await?;
    match repository.search_doctors(term).await {
        Ok(hits) => Ok(hits),
        Err(err) => {
            note_query_outcome(&state, &err).await;
            Err(err.into())
        }
    }
}

#[tauri::command]
pub async fn fetch_usage(
    period: RequisitionPeriod,
    state: State<'_, AppState>,
) -> Result<Vec<UsageRow>, CommandError> {
    require_session(&state).await?;
    validate_period(&period)?;
    let settings = state.settings.lock().await.clone();
    if settings.catalog.is_empty() || settings.doctors.is_empty() {
        return Ok(Vec::new());
    }
    let icodes = settings.catalog.icodes();
    let doctor_codes: Vec<String> = settings
        .doctors
        .iter()
        .map(|doctor| doctor.code.clone())
        .collect();

    let repository = acquire_repository(&state).await?;
    match repository
        .fetch_usage(&icodes, &doctor_codes, &period)
        .await
    {
        Ok(rows) => {
            *state.health.lock().await = ConnectionHealth::Connected;
            Ok(rows)
        }
        Err(err) => {
            note_query_outcome(&state, &err).await;
            Err(err.into())
        }
    }
}

#[tauri::command]
pub async fn period_context(
    period: RequisitionPeriod,
    state: State<'_, AppState>,
) -> Result<PeriodContext, CommandError> {
    require_session(&state).await?;
    validate_period(&period)?;
    let records = state.requisitions.lock().await;
    Ok(PeriodContext {
        conflicts: existing_conflicts(&records, period),
    })
}

#[tauri::command]
pub async fn list_requisitions(
    state: State<'_, AppState>,
) -> Result<Vec<RequisitionRecord>, CommandError> {
    require_session(&state).await?;
    let mut records = state.requisitions.lock().await.clone();
    records.sort_by_key(|record| std::cmp::Reverse(record.printed_at));
    Ok(records)
}

#[tauri::command]
pub async fn save_and_lock_requisition(
    period: RequisitionPeriod,
    rows: Vec<ReportRow>,
    report: ReportMeta,
    state: State<'_, AppState>,
) -> Result<RequisitionRecord, CommandError> {
    let user = require_session(&state).await?;
    validate_period(&period)?;
    validate_report(&report)?;
    let rows: Vec<ReportRow> = rows.into_iter().filter(|row| row.requested > 0.0).collect();
    if rows.is_empty() {
        return Err(CommandError::validation("ไม่มีรายการยาในใบเบิก"));
    }
    let settings = state.settings.lock().await.clone();
    validate_rows(&settings.catalog, &rows)?;

    let mut records = state.requisitions.lock().await;
    let conflicts = existing_conflicts(&records, period);
    if !conflicts.is_empty() {
        let documents = conflicts
            .iter()
            .map(|record| record.document_no.clone())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(CommandError::locked(format!(
            "ช่วงวันที่นี้มีใบเบิกแล้ว (เลขที่ {documents}) ไม่สามารถแก้ไขหรือเบิกซ้ำได้"
        )));
    }

    let record = build_record(
        &records,
        period,
        rows,
        report,
        settings.approvals.clone(),
        user.display_name,
        Local::now().naive_local(),
    );
    let mut next = records.clone();
    next.push(record.clone());
    requisitions::save(&state.requisitions_path(), &next).await?;
    *records = next;
    Ok(record)
}

pub async fn run_health_monitor(state: AppState) {
    loop {
        let health = probe_health(&state).await;
        *state.health.lock().await = health;
        tokio::time::sleep(HEALTH_POLL_INTERVAL).await;
    }
}

async fn probe_health(state: &AppState) -> ConnectionHealth {
    if !state.config_path().exists() {
        return ConnectionHealth::Unconfigured;
    }
    let Ok(vault) = load_vault() else {
        return ConnectionHealth::Disconnected;
    };
    let Ok(Some(config)) =
        snap_herb_hosxp_connector::config::load(&state.config_path(), &vault).await
    else {
        return ConnectionHealth::Disconnected;
    };
    match ensure_pool(state, &config).await {
        Ok(pool) => match ConnectorRepository::new(pool).ping().await {
            Ok(()) => ConnectionHealth::Connected,
            Err(_) => {
                *state.pool.lock().await = None;
                ConnectionHealth::Disconnected
            }
        },
        Err(_) => ConnectionHealth::Disconnected,
    }
}

async fn ensure_pool(
    state: &AppState,
    config: &HosxConfig,
) -> Result<MySqlPool, snap_herb_hosxp_connector::Error> {
    let mut guard = state.pool.lock().await;
    if let Some(pool) = guard.as_ref() {
        return Ok(pool.clone());
    }
    let pool = connect_hosxp(config).await?;
    *guard = Some(pool.clone());
    Ok(pool)
}

async fn require_session(state: &AppState) -> Result<SessionUser, CommandError> {
    state
        .session
        .lock()
        .await
        .clone()
        .ok_or_else(CommandError::unauthorized)
}

async fn acquire_repository(state: &AppState) -> Result<ConnectorRepository, CommandError> {
    if !state.config_path().exists() {
        return Err(CommandError::not_configured());
    }
    let vault = load_vault().map_err(CommandError::from)?;
    let config = snap_herb_hosxp_connector::config::load(&state.config_path(), &vault)
        .await
        .map_err(CommandError::from)?
        .ok_or_else(CommandError::not_configured)?;
    let pool = ensure_pool(state, &config)
        .await
        .map_err(CommandError::from)?;
    Ok(ConnectorRepository::new(pool))
}

async fn update_settings(
    state: &AppState,
    mutate: impl FnOnce(&mut Settings),
) -> Result<(), CommandError> {
    let mut guard = state.settings.lock().await;
    let mut updated = guard.clone();
    mutate(&mut updated);
    settings::save(&state.settings_path(), &updated)
        .await
        .map_err(CommandError::from)?;
    *guard = updated;
    Ok(())
}

async fn note_query_outcome(state: &AppState, err: &RepositoryError) {
    if matches!(err, RepositoryError::Connection) {
        *state.pool.lock().await = None;
        *state.health.lock().await = ConnectionHealth::Disconnected;
    }
}

fn normalize_doctors(doctors: Vec<DoctorRef>) -> Result<Vec<DoctorRef>, CommandError> {
    let mut normalized = Vec::with_capacity(doctors.len());
    let mut seen = HashSet::new();
    for doctor in doctors {
        let code = doctor.code.trim();
        let name = doctor.name.trim();
        if code.is_empty() {
            return Err(CommandError::validation("รหัสแพทย์ต้องไม่ว่าง"));
        }
        if name.is_empty() {
            return Err(CommandError::validation("ชื่อแพทย์ต้องไม่ว่าง"));
        }
        if !seen.insert(code.to_string()) {
            return Err(CommandError::validation(format!("แพทย์รหัส {code} ถูกเลือกซ้ำ")));
        }
        normalized.push(DoctorRef::new(code, name));
    }
    Ok(normalized)
}

fn validate_catalog(catalog: &HerbCatalog) -> Result<(), CommandError> {
    let mut seen = HashSet::new();
    for drug in &catalog.drugs {
        if drug.icode.is_empty() {
            return Err(CommandError::validation("รหัสยาสมุนไพรต้องไม่ว่าง"));
        }
        if !seen.insert(drug.icode.clone()) {
            return Err(CommandError::validation(format!(
                "ยารหัส {} ถูกเลือกซ้ำ",
                drug.icode
            )));
        }
        if drug.name.is_empty() {
            return Err(CommandError::validation("ชื่อยาสมุนไพรต้องไม่ว่าง"));
        }
        if drug.usage_unit.is_empty() {
            return Err(CommandError::validation("หน่วยนับการใช้ต้องไม่ว่าง"));
        }
        if drug.pack_unit.is_empty() {
            return Err(CommandError::validation("หน่วยเบิกต้องไม่ว่าง"));
        }
        if !drug.units_per_pack.is_finite() || drug.units_per_pack <= 0.0 {
            return Err(CommandError::validation(format!(
                "จำนวนหน่วยใน 1 {} ของ {} ต้องมากกว่า 0",
                drug.pack_unit, drug.name
            )));
        }
        if !drug.max_packs.is_finite()
            || drug.max_packs < 0.0
            || drug.max_packs.fract().abs() > 1e-9
        {
            return Err(CommandError::validation(format!(
                "จำนวนสูงสุดที่เบิกได้ของ {} ต้องเป็นจำนวนเต็มและไม่ติดลบ",
                drug.name
            )));
        }
    }
    Ok(())
}

fn validate_rows(catalog: &HerbCatalog, rows: &[ReportRow]) -> Result<(), CommandError> {
    let mut seen = HashSet::new();
    for row in rows {
        if !seen.insert(row.icode.clone()) {
            return Err(CommandError::validation(format!(
                "รายการยารหัส {} ซ้ำในใบเบิก",
                row.icode
            )));
        }
        let Some(drug) = catalog.find(&row.icode) else {
            return Err(CommandError::validation(format!(
                "ไม่พบยารหัส {} ในบัญชีที่ตั้งค่าไว้",
                row.icode
            )));
        };
        if !row.usage.is_finite() || row.usage < 0.0 {
            return Err(CommandError::validation(format!(
                "จำนวนใช้ของ {} ไม่ถูกต้อง",
                row.name
            )));
        }
        if !row.requested.is_finite() || row.requested < 0.0 || row.requested.fract().abs() > 1e-9 {
            return Err(CommandError::validation(format!(
                "จำนวนเบิกของ {} ต้องเป็นจำนวนเต็มและไม่ติดลบ",
                row.name
            )));
        }
        if row.requested > drug.max_packs.floor() + 1e-9 {
            return Err(CommandError::validation(format!(
                "จำนวนเบิกของ {} เกินจำนวนสูงสุดที่ตั้งไว้ ({} {})",
                row.name,
                snap_herb_models::format_qty(drug.max_packs),
                drug.pack_unit
            )));
        }
        if row.has_adjustment() && row.reason.trim().is_empty() {
            return Err(CommandError::validation(format!(
                "กรุณาระบุเหตุผลการแก้ไขรายการ {}",
                row.name
            )));
        }
    }
    Ok(())
}

fn normalize_approvals(mut approvals: ApprovalSettings) -> ApprovalSettings {
    approvals.work_unit = approvals.work_unit.trim().to_string();
    approvals.requester_position = approvals.requester_position.trim().to_string();
    approvals.receiver_name = approvals.receiver_name.trim().to_string();
    approvals.receiver_position = approvals.receiver_position.trim().to_string();
    approvals.orderer_name = approvals.orderer_name.trim().to_string();
    approvals.orderer_position = approvals.orderer_position.trim().to_string();
    approvals.payer_name = approvals.payer_name.trim().to_string();
    approvals.payer_position = approvals.payer_position.trim().to_string();
    approvals.supervisor_name = approvals.supervisor_name.trim().to_string();
    approvals
}

fn validate_report(report: &ReportMeta) -> Result<(), CommandError> {
    if report.hospital_name.trim().is_empty() {
        return Err(CommandError::validation("กรุณากรอกชื่อโรงพยาบาล"));
    }
    if report.document_title.trim().is_empty() {
        return Err(CommandError::validation("กรุณากรอกชื่อเอกสาร"));
    }
    Ok(())
}

fn validate_period(period: &RequisitionPeriod) -> Result<(), CommandError> {
    validate_period_on(period, Local::now().date_naive())
}

fn validate_period_on(
    period: &RequisitionPeriod,
    today: chrono::NaiveDate,
) -> Result<(), CommandError> {
    RequisitionPeriod::new(period.start_date, period.end_date)
        .map_err(|_| CommandError::validation("ช่วงวันที่: วันที่เริ่มต้องไม่หลังวันที่สิ้นสุด"))?;
    if !period.ends_before(today) {
        return Err(CommandError::validation(
            "เลือกช่วงวันที่ได้ถึงวันที่ก่อนหน้าวันที่ปัจจุบันเท่านั้น",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    fn input(host: &str, port: u16, database: &str, user: &str) -> ConnectionInput {
        ConnectionInput {
            host: host.to_string(),
            port,
            database: database.to_string(),
            user: user.to_string(),
            password: "pw".to_string(),
        }
    }

    #[test]
    fn connection_input_validates_required_fields() {
        assert!(
            input("10.0.0.1", 3306, "hos", "snapherb_ro")
                .validate()
                .is_ok()
        );
        assert!(input(" ", 3306, "hos", "snapherb_ro").validate().is_err());
        assert!(
            input("10.0.0.1", 0, "hos", "snapherb_ro")
                .validate()
                .is_err()
        );
        assert!(
            input("10.0.0.1", 3306, " ", "snapherb_ro")
                .validate()
                .is_err()
        );
        assert!(input("10.0.0.1", 3306, "hos", " ").validate().is_err());
    }

    #[test]
    fn doctor_validation_trims_and_rejects_duplicates() {
        let doctors = normalize_doctors(vec![
            DoctorRef::new(" D01 ", " สมชาย "),
            DoctorRef::new("D02", "สมหญิง"),
        ])
        .expect("valid doctors");
        assert_eq!(doctors[0].code, "D01");
        assert_eq!(doctors[0].name, "สมชาย");
        assert!(
            normalize_doctors(vec![
                DoctorRef::new("D01", "สมชาย"),
                DoctorRef::new("D01", "ซ้ำ")
            ])
            .is_err()
        );
    }

    #[test]
    fn period_validation_rejects_today_and_future() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 24).expect("date");
        let yesterday = RequisitionPeriod::new(
            chrono::NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"),
            chrono::NaiveDate::from_ymd_opt(2026, 10, 23).expect("date"),
        )
        .expect("period");
        let today_period = RequisitionPeriod::new(
            chrono::NaiveDate::from_ymd_opt(2026, 10, 24).expect("date"),
            chrono::NaiveDate::from_ymd_opt(2026, 10, 24).expect("date"),
        )
        .expect("period");
        assert!(validate_period_on(&yesterday, today).is_ok());
        assert!(validate_period_on(&today_period, today).is_err());
    }

    #[test]
    fn catalog_validation_rejects_bad_pack_numbers() {
        let mut catalog = HerbCatalog::default();
        catalog.drugs.push(snap_herb_models::HerbDrug {
            icode: "1001".into(),
            name: "ขมิ้นชัน".into(),
            strength_text: String::new(),
            usage_unit: "เม็ด".into(),
            pack_unit: "กล่อง".into(),
            units_per_pack: 0.0,
            max_packs: 30.0,
            sort_order: 0,
        });
        assert!(validate_catalog(&catalog).is_err());
    }

    #[tokio::test]
    async fn require_session_rejects_without_a_session() {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = AppState::new(dir.path().to_path_buf(), AppSettings::default(), Vec::new());
        let error = require_session(&state).await.expect_err("must reject");
        assert_eq!(error.kind, crate::error::CommandErrorKind::Auth);
    }
}
