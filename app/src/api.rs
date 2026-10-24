use std::cell::RefCell;
use std::collections::HashMap;

use js_sys::{Object, Reflect};
use serde_wasm_bindgen::{from_value, to_value};
use snap_herb_models::{
    AppSettings, ApprovalSettings, ConnectionSummary, DoctorRef, DrugItem, HerbCatalog,
    PeriodContext, ReportMeta, ReportRow, RequisitionPeriod, RequisitionRecord, SessionUser,
    UsageRow,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::state::ConnectionHealth;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI_INTERNALS__"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
}

type MockInvoke = Box<dyn Fn(&str, JsValue) -> js_sys::Promise>;

thread_local! {
    static MOCK_INVOKE: RefCell<Option<MockInvoke>> = RefCell::new(None);
}

pub fn install_mock_invoke(mock: impl Fn(&str, JsValue) -> js_sys::Promise + 'static) {
    MOCK_INVOKE.with(|m| *m.borrow_mut() = Some(Box::new(mock)));
}

pub fn clear_mock_invoke() {
    MOCK_INVOKE.with(|m| *m.borrow_mut() = None);
}

fn call_invoke(cmd: &str, args: JsValue) -> js_sys::Promise {
    MOCK_INVOKE.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock(cmd, args)
        } else {
            invoke(cmd, args)
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiErrorKind {
    NotConfigured,
    Connection,
    Guard,
    Query,
    Validation,
    Auth,
    Storage,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub kind: ApiErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub latency_ms: u64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettingsInput {
    pub contents: String,
    pub file_name: String,
}

fn request_build_error() -> ApiError {
    ApiError {
        kind: ApiErrorKind::Query,
        message: "สร้างคำขอไม่สำเร็จ".to_string(),
    }
}

async fn call_empty<T>(cmd: &str) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let args = to_value(&HashMap::<String, String>::new()).map_err(|_| request_build_error())?;
    call_raw(cmd, args).await
}

async fn call_one<T>(cmd: &str, name: &str, arg: &impl serde::Serialize) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let value = to_value(arg).map_err(|_| request_build_error())?;
    let args = Object::new();
    Reflect::set(&args, &JsValue::from_str(name), &value).map_err(|_| request_build_error())?;
    call_raw(cmd, args.into()).await
}

async fn call_many<T>(cmd: &str, args: &[(&str, JsValue)]) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let object = Object::new();
    for (name, value) in args {
        Reflect::set(&object, &JsValue::from_str(name), value)
            .map_err(|_| request_build_error())?;
    }
    call_raw(cmd, object.into()).await
}

async fn call_raw<T>(cmd: &str, args: JsValue) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let promise = call_invoke(cmd, args);
    let value = JsFuture::from(promise).await.map_err(|err| {
        from_value::<ApiError>(err).unwrap_or(ApiError {
            kind: ApiErrorKind::Query,
            message: "การเชื่อมต่อกับโปรแกรมหลักล้มเหลว".to_string(),
        })
    })?;
    from_value(value).map_err(|_| ApiError {
        kind: ApiErrorKind::Query,
        message: "อ่านผลลัพธ์ไม่สำเร็จ".to_string(),
    })
}

pub async fn connection_status() -> bool {
    call_empty::<bool>("connection_status")
        .await
        .unwrap_or(false)
}

pub async fn connection_health() -> Result<ConnectionHealth, ApiError> {
    call_empty("connection_health").await
}

pub async fn configure_connection(input: &ConnectionInput) -> Result<(), ApiError> {
    call_one("configure_connection", "input", input).await
}

pub async fn test_connection(input: &ConnectionInput) -> Result<u64, ApiError> {
    let result: ConnectionTestResult = call_one("test_connection", "input", input).await?;
    Ok(result.latency_ms)
}

pub async fn connection_summary() -> Result<Option<ConnectionSummary>, ApiError> {
    call_empty("connection_summary").await
}

pub async fn connect(input: &ConnectionInput) -> Result<u64, ApiError> {
    let result: ConnectionTestResult = call_one("connect", "input", input).await?;
    Ok(result.latency_ms)
}

pub async fn reconnect() -> Result<u64, ApiError> {
    let result: ConnectionTestResult = call_empty("reconnect").await?;
    Ok(result.latency_ms)
}

pub async fn login(username: &str, password: &str) -> Result<SessionUser, ApiError> {
    call_one(
        "login",
        "input",
        &LoginInput {
            username: username.to_string(),
            password: password.to_string(),
        },
    )
    .await
}

pub async fn logout() -> Result<(), ApiError> {
    call_empty("logout").await
}

pub async fn current_session() -> Result<Option<SessionUser>, ApiError> {
    call_empty("current_session").await
}

pub async fn load_settings() -> Result<AppSettings, ApiError> {
    call_empty("load_settings").await
}

pub async fn save_doctors(doctors: &[DoctorRef]) -> Result<(), ApiError> {
    call_one("save_doctors", "doctors", &doctors).await
}

pub async fn save_catalog(catalog: &HerbCatalog) -> Result<(), ApiError> {
    call_one("save_catalog", "catalog", catalog).await
}

pub async fn save_report(report: &ReportMeta) -> Result<(), ApiError> {
    call_one("save_report", "report", report).await
}

pub async fn save_approvals(approvals: &ApprovalSettings) -> Result<(), ApiError> {
    call_one("save_approvals", "approvals", approvals).await
}

pub async fn export_settings_to_file(
    contents: &str,
    file_name: &str,
) -> Result<Option<String>, ApiError> {
    call_one(
        "export_settings_to_file",
        "input",
        &ExportSettingsInput {
            contents: contents.to_string(),
            file_name: file_name.to_string(),
        },
    )
    .await
}

pub async fn search_drugs(term: &str) -> Result<Vec<DrugItem>, ApiError> {
    call_one("search_drugs", "term", &term).await
}

pub async fn search_doctors(term: &str) -> Result<Vec<DoctorRef>, ApiError> {
    call_one("search_doctors", "term", &term).await
}

pub async fn fetch_usage(period: &RequisitionPeriod) -> Result<Vec<UsageRow>, ApiError> {
    call_one("fetch_usage", "period", period).await
}

pub async fn period_context(period: &RequisitionPeriod) -> Result<PeriodContext, ApiError> {
    call_one("period_context", "period", period).await
}

pub async fn list_requisitions() -> Result<Vec<RequisitionRecord>, ApiError> {
    call_empty("list_requisitions").await
}

pub async fn save_and_lock_requisition(
    period: &RequisitionPeriod,
    rows: &[ReportRow],
    report: &ReportMeta,
) -> Result<RequisitionRecord, ApiError> {
    let period = to_value(period).map_err(|_| request_build_error())?;
    let rows = to_value(rows).map_err(|_| request_build_error())?;
    let report = to_value(report).map_err(|_| request_build_error())?;
    call_many(
        "save_and_lock_requisition",
        &[("period", period), ("rows", rows), ("report", report)],
    )
    .await
}
