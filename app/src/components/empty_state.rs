use leptos::prelude::*;

use crate::components::icons::{IconClipboardList, IconLeaf, IconStethoscope};
use crate::state::{AppState, SettingsTab};

#[component]
pub fn CatalogEmpty(state: AppState) -> impl IntoView {
    let open = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Catalog);
            state.settings_open.set(true);
        }
    };
    view! {
        <div class="empty-card">
            <span class="empty-card__icon">
                <IconLeaf class="icon icon--large" />
            </span>
            <h2 class="empty-card__title">"ยังไม่มีรายการยาสมุนไพร"</h2>
            <p class="empty-card__text">
                "เพิ่มยาสมุนไพรจากบัญชียา HOSxP พร้อมกำหนดหน่วยเบิก จำนวนหน่วยนับใน 1 หน่วยเบิก และจำนวนสูงสุดต่อรอบ"
            </p>
            <button class="button-accent" on:click=open>
                "ไปที่ตั้งค่ายาสมุนไพร"
            </button>
        </div>
    }
}

#[component]
pub fn DoctorsEmpty(state: AppState) -> impl IntoView {
    let open = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Doctors);
            state.settings_open.set(true);
        }
    };
    view! {
        <div class="empty-card">
            <span class="empty-card__icon">
                <IconStethoscope class="icon icon--large" />
            </span>
            <h2 class="empty-card__title">"ยังไม่ได้เลือกแพทย์ที่ต้องการตรวจจับ"</h2>
            <p class="empty-card__text">
                "เลือกแพทย์ผู้สั่งจ่ายจาก HOSxP เพื่อให้ระบบนับเฉพาะรายการยาที่แพทย์เหล่านั้นสั่ง"
            </p>
            <button class="button-accent" on:click=open>
                "ไปที่ตั้งค่าแพทย์"
            </button>
        </div>
    }
}

#[component]
pub fn NoDataHint(state: AppState) -> impl IntoView {
    let loading = state.loading;
    view! {
        <div class="empty-card empty-card--quiet">
            <span class="empty-card__icon">
                <IconClipboardList class="icon icon--large" />
            </span>
            <h2 class="empty-card__title">
                {move || {
                    if loading.get() { "กำลังดึงข้อมูล..." } else { "พร้อมสร้างใบเบิก" }
                }}
            </h2>
            <p class="empty-card__text">
                {move || {
                    if loading.get() {
                        "กำลังอ่านยอดใช้ยาสมุนไพรตามแพทย์และช่วงวันที่ที่เลือก"
                    } else {
                        "เลือกช่วงวันที่ด้านบน แล้วกด \"ดึงข้อมูลการใช้ยา\" เพื่อเริ่มสร้างใบเบิก"
                    }
                }}
            </p>
        </div>
    }
}
