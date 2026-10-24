use std::time::Duration;

use leptos::prelude::*;
use snap_herb_core::CatalogOpsError;
use snap_herb_models::DrugItem;

use crate::api;
use crate::components::icons::{
    IconChevronDown, IconChevronUp, IconLeaf, IconPlus, IconSearch, IconTrash,
};
use crate::components::settings_modal::{StatusLine, StatusMessage};
use crate::state::AppState;

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(250);
const MIN_SEARCH_CHARS: usize = 3;

fn catalog_error_th(err: &CatalogOpsError) -> String {
    match err {
        CatalogOpsError::EmptyName => "กรุณากรอกชื่อ".to_string(),
        CatalogOpsError::EmptyCode => "รหัสยาต้องไม่ว่าง".to_string(),
        CatalogOpsError::EmptyUnit => "กรุณากรอกหน่วยนับและหน่วยเบิก".to_string(),
        CatalogOpsError::InvalidUnitsPerPack => "จำนวนหน่วยนับใน 1 หน่วยเบิกต้องมากกว่า 0".to_string(),
        CatalogOpsError::InvalidMaxPacks => "จำนวนสูงสุดที่เบิกได้ต้องเป็นจำนวนเต็มและไม่ติดลบ".to_string(),
        CatalogOpsError::DuplicateDrug(icode) => format!("ยารหัส {icode} อยู่ในบัญชีแล้ว"),
        CatalogOpsError::DuplicateDoctor(code) => format!("แพทย์รหัส {code} ถูกเลือกแล้ว"),
        CatalogOpsError::DrugNotFound(_) => "ไม่พบรายการยา".to_string(),
        CatalogOpsError::DoctorNotFound(_) => "ไม่พบแพทย์".to_string(),
    }
}

#[component]
pub fn CatalogPanel(state: AppState) -> impl IntoView {
    let draft = RwSignal::new(
        state
            .settings
            .get_untracked()
            .map(|settings| settings.catalog)
            .unwrap_or_default(),
    );
    let term = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<DrugItem>::new());
    let searching = RwSignal::new(false);
    let search_seq = RwSignal::new(0_u64);
    let expanded = RwSignal::new(None::<String>);
    let manual_open = RwSignal::new(false);
    let manual_name = RwSignal::new(String::new());
    let manual_strength = RwSignal::new(String::new());
    let manual_usage_unit = RwSignal::new("หน่วย".to_string());
    let manual_pack_unit = RwSignal::new("กล่อง".to_string());
    let manual_units = RwSignal::new("1".to_string());
    let manual_max = RwSignal::new("1".to_string());
    let status = RwSignal::new(None::<StatusMessage>);
    let busy = RwSignal::new(false);
    let dirty = RwSignal::new(false);

    let on_term_input = move |ev| {
        let raw = event_target_value(&ev);
        term.set(raw.clone());
        let seq = search_seq.get_untracked().wrapping_add(1);
        search_seq.set(seq);
        results.set(Vec::new());
        status.set(None);
        let needle = raw.trim().to_string();
        if needle.chars().count() < MIN_SEARCH_CHARS {
            searching.set(false);
            return;
        }
        searching.set(true);
        let _ = set_timeout_with_handle(
            move || {
                if search_seq.get_untracked() != seq {
                    return;
                }
                leptos::task::spawn_local(async move {
                    let outcome = api::search_drugs(&needle).await;
                    if search_seq.get_untracked() != seq {
                        return;
                    }
                    searching.set(false);
                    match outcome {
                        Ok(hits) => results.set(hits),
                        Err(err) => {
                            results.set(Vec::new());
                            status.set(Some(StatusMessage::error(err.message)));
                        }
                    }
                });
            },
            SEARCH_DEBOUNCE,
        );
    };

    let on_add_drug = move |item: DrugItem| {
        let usage_unit = if item.units.trim().is_empty() {
            "หน่วย".to_string()
        } else {
            item.units.trim().to_string()
        };
        let mut error = None;
        draft.update(|catalog| {
            if let Err(err) =
                snap_herb_core::add_herb(catalog, &item, &usage_unit, "กล่อง", 1.0, 1.0)
            {
                error = Some(catalog_error_th(&err));
            }
        });
        match error {
            Some(message) => status.set(Some(StatusMessage::error(message))),
            None => {
                dirty.set(true);
                expanded.set(Some(item.icode.clone()));
                status.set(Some(StatusMessage::ok(format!(
                    "เพิ่ม {} แล้ว กรุณากำหนดหน่วยเบิกและจำนวนสูงสุดก่อนบันทึก",
                    item.name
                ))));
            }
        }
    };

    let on_manual_add = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = manual_name.get_untracked().trim().to_string();
        if name.is_empty() {
            status.set(Some(StatusMessage::error("กรุณากรอกชื่อยา".to_string())));
            return;
        }
        let units_per_pack = match manual_units.get_untracked().trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                status.set(Some(StatusMessage::error(
                    "จำนวนหน่วยนับใน 1 หน่วยเบิกต้องเป็นตัวเลข".to_string(),
                )));
                return;
            }
        };
        let max_packs = match manual_max.get_untracked().trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                status.set(Some(StatusMessage::error(
                    "จำนวนสูงสุดที่เบิกได้ต้องเป็นตัวเลข".to_string(),
                )));
                return;
            }
        };
        let usage_unit = manual_usage_unit.get_untracked().trim().to_string();
        let pack_unit = manual_pack_unit.get_untracked().trim().to_string();
        let strength = manual_strength.get_untracked().trim().to_string();
        let existing = draft.get_untracked();
        let mut next = existing
            .drugs
            .iter()
            .filter_map(|drug| {
                drug.icode
                    .strip_prefix("MANUAL-")
                    .and_then(|suffix| suffix.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(0)
            + 1;
        let icode = loop {
            let candidate = format!("MANUAL-{next:03}");
            if !existing.drugs.iter().any(|drug| drug.icode == candidate) {
                break candidate;
            }
            next += 1;
        };
        let item = DrugItem {
            icode,
            name: name.clone(),
            strength,
            units: String::new(),
        };
        let mut error = None;
        draft.update(|catalog| {
            if let Err(err) = snap_herb_core::add_herb(
                catalog,
                &item,
                &usage_unit,
                &pack_unit,
                units_per_pack,
                max_packs,
            ) {
                error = Some(catalog_error_th(&err));
            }
        });
        match error {
            Some(message) => status.set(Some(StatusMessage::error(message))),
            None => {
                dirty.set(true);
                expanded.set(Some(item.icode.clone()));
                manual_name.set(String::new());
                manual_strength.set(String::new());
                manual_usage_unit.set("หน่วย".to_string());
                manual_pack_unit.set("กล่อง".to_string());
                manual_units.set("1".to_string());
                manual_max.set("1".to_string());
                status.set(Some(StatusMessage::ok(format!(
                    "เพิ่ม {} แล้ว อย่าลืมกดบันทึกรายการยา",
                    item.name
                ))));
            }
        }
    };

    let on_update = move |icode: String,
                          name: String,
                          strength: String,
                          usage_unit: String,
                          pack_unit: String,
                          units_text: String,
                          max_text: String| {
        let units_per_pack = match units_text.trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                status.set(Some(StatusMessage::error(
                    "จำนวนหน่วยนับใน 1 หน่วยเบิกต้องเป็นตัวเลข".to_string(),
                )));
                return;
            }
        };
        let max_packs = match max_text.trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                status.set(Some(StatusMessage::error(
                    "จำนวนสูงสุดที่เบิกได้ต้องเป็นตัวเลข".to_string(),
                )));
                return;
            }
        };
        let mut error = None;
        draft.update(|catalog| {
            if let Err(err) = snap_herb_core::update_herb(
                catalog,
                &icode,
                &name,
                &strength,
                &usage_unit,
                &pack_unit,
                units_per_pack,
                max_packs,
            ) {
                error = Some(catalog_error_th(&err));
            }
        });
        match error {
            Some(message) => status.set(Some(StatusMessage::error(message))),
            None => {
                dirty.set(true);
                status.set(None);
            }
        }
    };

    let on_move = move |icode: String, delta: i32| {
        let mut error = None;
        draft.update(|catalog| {
            if let Err(err) = snap_herb_core::move_herb(catalog, &icode, delta) {
                error = Some(catalog_error_th(&err));
            }
        });
        if let Some(message) = error {
            status.set(Some(StatusMessage::error(message)));
        } else {
            dirty.set(true);
            status.set(None);
        }
    };

    let on_remove = move |icode: String| {
        let mut error = None;
        draft.update(|catalog| {
            if let Err(err) = snap_herb_core::remove_herb(catalog, &icode) {
                error = Some(catalog_error_th(&err));
            }
        });
        if let Some(message) = error {
            status.set(Some(StatusMessage::error(message)));
        } else {
            if expanded.get_untracked().as_deref() == Some(icode.as_str()) {
                expanded.set(None);
            }
            dirty.set(true);
            status.set(None);
        }
    };

    let on_save = {
        let state = state.clone();
        move |_| {
            let catalog = draft.get_untracked();
            busy.set(true);
            status.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::save_catalog(&catalog).await {
                    Ok(()) => {
                        state.settings.update(|settings| {
                            if let Some(settings) = settings {
                                settings.catalog = catalog;
                            }
                        });
                        state.clear_requisition();
                        dirty.set(false);
                        status.set(Some(StatusMessage::ok("บันทึกรายการยาสมุนไพรแล้ว")));
                    }
                    Err(err) => status.set(Some(StatusMessage::error(err.message))),
                }
                busy.set(false);
            });
        }
    };

    let on_reset = {
        let state = state.clone();
        move |_| {
            draft.set(
                state
                    .settings
                    .get_untracked()
                    .map(|settings| settings.catalog)
                    .unwrap_or_default(),
            );
            results.set(Vec::new());
            term.set(String::new());
            searching.set(false);
            search_seq.update(|seq| *seq = seq.wrapping_add(1));
            expanded.set(None);
            dirty.set(false);
            status.set(None);
        }
    };

    view! {
        <div class="panel panel--catalog">
            <div class="panel-head">
                <span class="panel-head__icon">
                    <IconLeaf class="icon" />
                </span>
                <div>
                    <h3 class="panel__subtitle">"รายการยาสมุนไพรที่ต้องการตรวจจับ"</h3>
                    <p class="panel__note">
                        "กำหนดหน่วยนับ หน่วยเบิก จำนวนหน่วยนับใน 1 หน่วยเบิก และจำนวนสูงสุดที่เบิกได้ต่อรอบ"
                    </p>
                </div>
            </div>

            <div class="picker">
                <span class="panel__label">"ค้นหายาสมุนไพรจาก HOSxP"</span>
                <div class="picker__search">
                    <span class="picker__icon">
                        <IconSearch class="icon" />
                    </span>
                    <input
                        class="text-input"
                        type="search"
                        placeholder="พิมพ์ชื่อหรือรหัสยา อย่างน้อย 3 ตัวอักษร"
                        prop:value=move || term.get()
                        on:input=on_term_input
                    />
                </div>
                <p class="panel__note">
                    {move || {
                        let count = term.get().trim().chars().count();
                        if searching.get() {
                            "กำลังค้นหายา...".to_string()
                        } else if count == 0 {
                            String::new()
                        } else if count < MIN_SEARCH_CHARS {
                            format!("พิมพ์อีก {} ตัวอักษร", MIN_SEARCH_CHARS - count)
                        } else if results.get().is_empty() {
                            "ไม่พบยาที่ตรงกับคำค้น".to_string()
                        } else {
                            format!("พบยา {} รายการ", results.get().len())
                        }
                    }}
                </p>
                <ul class="picker__results">
                    {move || {
                        results
                            .get()
                            .into_iter()
                            .map(|item| {
                                let selected = draft
                                    .get()
                                    .drugs
                                    .iter()
                                    .any(|drug| drug.icode == item.icode);
                                let add_item = item.clone();
                                let strength = item.strength_label();
                                view! {
                                    <li class="picker__result">
                                        <div class="picker__result-info">
                                            <span class="picker__result-name">
                                                {item.name.clone()}
                                            </span>
                                            <span class="picker__result-meta">
                                                <span class="badge-icode">
                                                    {item.icode.clone()}
                                                </span>
                                                {(!strength.is_empty())
                                                    .then(|| {
                                                        view! {
                                                            <span class="picker__result-strength">
                                                                {strength.clone()}
                                                            </span>
                                                        }
                                                    })}
                                                {(!item.units.trim().is_empty())
                                                    .then(|| {
                                                        view! {
                                                            <span class="picker__result-strength">
                                                                {format!("หน่วยยา {}", item.units)}
                                                            </span>
                                                        }
                                                    })}
                                            </span>
                                        </div>
                                        <button
                                            class="button-ghost button-ghost--inline"
                                            type="button"
                                            disabled=selected
                                            on:click=move |_| on_add_drug(add_item.clone())
                                        >
                                            <IconPlus class="icon" />
                                            {if selected { "เลือกแล้ว" } else { "เพิ่ม" }}
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </div>

            <div class="manual-herb">
                <button
                    class="button-secondary manual-herb__toggle"
                    type="button"
                    on:click=move |_| manual_open.update(|open| *open = !*open)
                >
                    <IconPlus class="icon" />
                    {move || {
                        if manual_open.get() {
                            "ซ่อนการเพิ่มรายการเอง"
                        } else {
                            "เพิ่มรายการที่ต้องการเบิกด้วยตนเอง"
                        }
                    }}
                </button>
                {move || {
                    manual_open
                        .get()
                        .then(|| {
                            view! {
                                <form class="form-card manual-herb__form" on:submit=on_manual_add>
                <div>
                    <h3 class="panel__subtitle">"เพิ่มรายการที่ต้องการเบิกด้วยตนเอง"</h3>
                    <p class="panel__note">
                        "สำหรับรายการที่ไม่มีใน HOSxP และจะไม่ถูกดึงยอดใช้จาก HOSxP"
                    </p>
                </div>
                <div class="manual-herb__grid">
                    <label class="field">
                        <span class="field__label">"ชื่อยา"</span>
                        <input
                            class="text-input"
                            type="text"
                            placeholder="เช่น ยาแก้ลมแก้เส้น"
                            prop:value=move || manual_name.get()
                            on:input=move |ev| manual_name.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"ความแรง"</span>
                        <input
                            class="text-input"
                            type="text"
                            placeholder="เช่น 500 mg"
                            prop:value=move || manual_strength.get()
                            on:input=move |ev| manual_strength.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"หน่วยนับการใช้"</span>
                        <input
                            class="text-input"
                            type="text"
                            placeholder="เช่น แคปซูล"
                            prop:value=move || manual_usage_unit.get()
                            on:input=move |ev| manual_usage_unit.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"หน่วยเบิก"</span>
                        <input
                            class="text-input"
                            type="text"
                            placeholder="เช่น กล่อง"
                            prop:value=move || manual_pack_unit.get()
                            on:input=move |ev| manual_pack_unit.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"จำนวนหน่วยนับใน 1 หน่วยเบิก"</span>
                        <input
                            class="text-input"
                            type="number"
                            min="0.0001"
                            step="any"
                            prop:value=move || manual_units.get()
                            on:input=move |ev| manual_units.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="field">
                        <span class="field__label">"จำนวนสูงสุด/รอบ"</span>
                        <input
                            class="text-input"
                            type="number"
                            min="0"
                            step="1"
                            prop:value=move || manual_max.get()
                            on:input=move |ev| manual_max.set(event_target_value(&ev))
                        />
                    </label>
                </div>
                <div class="panel__actions">
                    <button class="button-secondary" type="submit">
                        <IconPlus class="icon" />
                        "เพิ่มรายการ"
                    </button>
                </div>
                                </form>
                            }
                        })
                }}
            </div>

            {move || {
                let catalog = draft.get();
                let drugs = catalog.drugs_in_order();
                if drugs.is_empty() {
                    return view! {
                        <div class="empty-inline">
                            <IconLeaf class="icon icon--large" />
                            <p>"ยังไม่มีรายการยาสมุนไพร ค้นหาและเพิ่มรายการด้านบน"</p>
                        </div>
                    }
                        .into_any();
                }
                drugs
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|drug| {
                        let icode_name = drug.icode.clone();
                        let icode_strength = drug.icode.clone();
                        let icode_usage = drug.icode.clone();
                        let icode_pack = drug.icode.clone();
                        let icode_units = drug.icode.clone();
                        let icode_max = drug.icode.clone();
                        let icode_up = drug.icode.clone();
                        let icode_down = drug.icode.clone();
                        let icode_remove = drug.icode.clone();

                        let strength_for_name = drug.strength_text.clone();
                        let usage_for_name = drug.usage_unit.clone();
                        let pack_for_name = drug.pack_unit.clone();
                        let units_for_name = drug.units_per_pack.to_string();
                        let max_for_name = drug.max_packs.to_string();

                        let name_for_strength = drug.name.clone();
                        let usage_for_strength = drug.usage_unit.clone();
                        let pack_for_strength = drug.pack_unit.clone();
                        let units_for_strength = drug.units_per_pack.to_string();
                        let max_for_strength = drug.max_packs.to_string();

                        let name_for_usage = drug.name.clone();
                        let strength_for_usage = drug.strength_text.clone();
                        let pack_for_usage = drug.pack_unit.clone();
                        let units_for_usage = drug.units_per_pack.to_string();
                        let max_for_usage = drug.max_packs.to_string();

                        let name_for_pack = drug.name.clone();
                        let strength_for_pack = drug.strength_text.clone();
                        let usage_for_pack = drug.usage_unit.clone();
                        let units_for_pack = drug.units_per_pack.to_string();
                        let max_for_pack = drug.max_packs.to_string();

                        let name_for_units = drug.name.clone();
                        let strength_for_units = drug.strength_text.clone();
                        let usage_for_units = drug.usage_unit.clone();
                        let pack_for_units = drug.pack_unit.clone();
                        let max_for_units = drug.max_packs.to_string();

                        let name_for_max = drug.name.clone();
                        let strength_for_max = drug.strength_text.clone();
                        let usage_for_max = drug.usage_unit.clone();
                        let pack_for_max = drug.pack_unit.clone();
                        let units_for_max = drug.units_per_pack.to_string();
                        let is_expanded =
                            expanded.get().as_deref() == Some(drug.icode.as_str());
                        let toggle_icode = drug.icode.clone();

                        view! {
                            <div class="herb-row">
                                <button
                                    class="herb-row__toggle"
                                    type="button"
                                    aria-expanded=is_expanded.to_string()
                                    on:click=move |_| {
                                        if expanded.get_untracked().as_deref()
                                            == Some(toggle_icode.as_str())
                                        {
                                            expanded.set(None);
                                        } else {
                                            expanded.set(Some(toggle_icode.clone()));
                                        }
                                    }
                                >
                                    <span class="herb-row__title">{drug.name.clone()}</span>
                                    {if is_expanded {
                                        view! { <IconChevronUp class="icon" /> }.into_any()
                                    } else {
                                        view! { <IconChevronDown class="icon" /> }.into_any()
                                    }}
                                </button>
                                {is_expanded
                                    .then(|| {
                                        view! {
                                            <div class="herb-row__details">
                                <div class="herb-row__line">
                                    <input
                                        class="text-input herb-row__name"
                                        type="text"
                                        aria-label="ชื่อยา"
                                        value=drug.name.clone()
                                        on:change=move |ev| {
                                            on_update(
                                                icode_name.clone(),
                                                event_target_value(&ev),
                                                strength_for_name.clone(),
                                                usage_for_name.clone(),
                                                pack_for_name.clone(),
                                                units_for_name.clone(),
                                                max_for_name.clone(),
                                            )
                                        }
                                    />
                                    {if drug.icode.starts_with("MANUAL-") {
                                        view! {
                                            <span class="badge-icode badge-icode--manual">
                                                "เพิ่มเอง"
                                            </span>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <span class="badge-icode">{drug.icode.clone()}</span>
                                        }
                                            .into_any()
                                    }}
                                    <div class="herb-row__actions">
                                        <button
                                            class="button-icon-circular button-icon-circular--small"
                                            aria-label="เลื่อนขึ้น"
                                            on:click=move |_| on_move(icode_up.clone(), -1)
                                        >
                                            <IconChevronUp class="icon" />
                                        </button>
                                        <button
                                            class="button-icon-circular button-icon-circular--small"
                                            aria-label="เลื่อนลง"
                                            on:click=move |_| on_move(icode_down.clone(), 1)
                                        >
                                            <IconChevronDown class="icon" />
                                        </button>
                                        <button
                                            class="button-icon-circular button-icon-circular--small"
                                            aria-label="ลบรายการยา"
                                            on:click=move |_| on_remove(icode_remove.clone())
                                        >
                                            <IconTrash class="icon" />
                                        </button>
                                    </div>
                                </div>
                                <div class="herb-row__line herb-row__line--fields">
                                    <label class="herb-row__cell">
                                        <span class="herb-row__cell-label">"ความแรง"</span>
                                        <input
                                            class="text-input"
                                            type="text"
                                            placeholder="เช่น 500 mg"
                                            value=drug.strength_text.clone()
                                            on:change=move |ev| {
                                                on_update(
                                                    icode_strength.clone(),
                                                    name_for_strength.clone(),
                                                    event_target_value(&ev),
                                                    usage_for_strength.clone(),
                                                    pack_for_strength.clone(),
                                                    units_for_strength.clone(),
                                                    max_for_strength.clone(),
                                                )
                                            }
                                        />
                                    </label>
                                    <label class="herb-row__cell">
                                        <span class="herb-row__cell-label">"หน่วยนับการใช้"</span>
                                        <input
                                            class="text-input"
                                            type="text"
                                            placeholder="เช่น เม็ด"
                                            value=drug.usage_unit.clone()
                                            on:change=move |ev| {
                                                on_update(
                                                    icode_usage.clone(),
                                                    name_for_usage.clone(),
                                                    strength_for_usage.clone(),
                                                    event_target_value(&ev),
                                                    pack_for_usage.clone(),
                                                    units_for_usage.clone(),
                                                    max_for_usage.clone(),
                                                )
                                            }
                                        />
                                    </label>
                                    <label class="herb-row__cell">
                                        <span class="herb-row__cell-label">"หน่วยเบิก"</span>
                                        <input
                                            class="text-input"
                                            type="text"
                                            placeholder="เช่น กล่อง"
                                            value=drug.pack_unit.clone()
                                            on:change=move |ev| {
                                                on_update(
                                                    icode_pack.clone(),
                                                    name_for_pack.clone(),
                                                    strength_for_pack.clone(),
                                                    usage_for_pack.clone(),
                                                    event_target_value(&ev),
                                                    units_for_pack.clone(),
                                                    max_for_pack.clone(),
                                                )
                                            }
                                        />
                                    </label>
                                </div>
                                <div class="herb-row__line herb-row__line--fields">
                                    <label class="herb-row__cell">
                                        <span class="herb-row__cell-label">
                                            {format!("จำนวนหน่วยนับใน 1 {}", drug.pack_unit)}
                                        </span>
                                        <input
                                            class="text-input"
                                            type="number"
                                            min="0.0001"
                                            step="any"
                                            value=drug.units_per_pack.to_string()
                                            on:change=move |ev| {
                                                on_update(
                                                    icode_units.clone(),
                                                    name_for_units.clone(),
                                                    strength_for_units.clone(),
                                                    usage_for_units.clone(),
                                                    pack_for_units.clone(),
                                                    event_target_value(&ev),
                                                    max_for_units.clone(),
                                                )
                                            }
                                        />
                                    </label>
                                    <label class="herb-row__cell">
                                        <span class="herb-row__cell-label">
                                            {format!("จำนวนสูงสุด/รอบ ({})", drug.pack_unit)}
                                        </span>
                                        <input
                                            class="text-input"
                                            type="number"
                                            min="0"
                                            step="1"
                                            value=drug.max_packs.to_string()
                                            on:change=move |ev| {
                                                on_update(
                                                    icode_max.clone(),
                                                    name_for_max.clone(),
                                                    strength_for_max.clone(),
                                                    usage_for_max.clone(),
                                                    pack_for_max.clone(),
                                                    units_for_max.clone(),
                                                    event_target_value(&ev),
                                                )
                                            }
                                        />
                                    </label>
                                    <div class="herb-row__summary">
                                        <span class="herb-row__summary-line">
                                            {drug.pack_label()}
                                        </span>
                                        <span class="herb-row__summary-line herb-row__summary-line--muted">
                                            {format!("เบิกได้สูงสุด {}", drug.max_label())}
                                        </span>
                                    </div>
                                </div>
                                            </div>
                                        }
                                    })}
                            </div>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}

            <StatusLine message=status />
            <div class="panel__actions">
                <span class="panel__dirty">
                    {move || dirty.get().then_some("มีการแก้ไขที่ยังไม่บันทึก")}
                </span>
                <button
                    class="button-secondary"
                    disabled=move || busy.get() || !dirty.get()
                    on:click=on_reset
                >
                    "ยกเลิกการแก้ไข"
                </button>
                <button
                    class="button-accent"
                    disabled=move || busy.get() || !dirty.get()
                    on:click=on_save
                >
                    "บันทึกรายการยา"
                </button>
            </div>
        </div>
    }
}
