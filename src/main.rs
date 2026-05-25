mod table;
mod backend;
mod metric_list;
mod agent;
mod sort_button;
mod add_metrics;

use std::future::Future;
use std::pin::Pin;
use leptos::prelude::*;
use leptos::server::codee::string::FromToStringCodec;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_cookie;
use table::{CellData, DenseTable};
use crate::add_metrics::AddMetrics;
use crate::agent::AiChat;
use crate::backend::{ai_agent_turn, pull_from_backend, MetricType};
use crate::metric_list::MetricList;

fn main() {
    console_error_panic_hook::set_once();

    let (metric_type, set_metric_type) = signal::<MetricType>(MetricType::LLM);
    let (session_id, set_session_id) = signal::<Option<String>>(None);
    let (cookie_refresh, set_cookie_refresh) = signal(0usize);
    let cookie_name = Signal::derive(move || format!("user_models_{:?}", metric_type.get()));

    mount_to(
        document()
            .get_element_by_id("table-mount")
            .unwrap()
            .unchecked_into(),
        move || view! {
            <TableLoader
                metric_type=metric_type
                trigger_refresh=cookie_refresh
                cookie_name=cookie_name
            />
        },
    ).forget();

    mount_to(
        document()
            .get_element_by_id("chat-mount")
            .unwrap()
            .unchecked_into(),
        move || view! {
            <AiChat on_send=Callback::new(move |(user_message, embed)| {
                let (cookie, _) = use_cookie::<
                    String,
                    FromToStringCodec,
                >(&cookie_name.get_untracked());
                let mt = metric_type.get_untracked();
                let session_id = session_id.get_untracked();
                let user_metrics = cookie.get_untracked().unwrap_or_default();
                Box::pin(async move {
                    let (response, id) = ai_agent_turn(
                            user_message,
                            embed,
                            mt,
                            session_id,
                            user_metrics,
                        )
                        .await;
                    set_session_id.set(Some(id));
                    response
                }) as Pin<Box<dyn Future<Output = String>>>
            }) />
        },
    ).forget();

    mount_to(
        document()
            .get_element_by_id("add-mount")
            .unwrap()
            .unchecked_into(),
        move || view! { <AddMetrics set_trigger_refresh=set_cookie_refresh /> },
    ).forget();

    mount_to(
        document()
            .get_element_by_id("list-mount")
            .unwrap()
            .unchecked_into(),
        move || view! { <MetricList metric_type=metric_type set_metric_type=set_metric_type /> },
    ).forget();
}

pub fn parse_csv(csv: &str) -> (Vec<CellData>, Vec<Vec<CellData>>) {
    let mut lines = csv.lines().filter(|l| !l.trim().is_empty());

    let headers: Vec<CellData> = lines.next().unwrap_or("")
        .split(',')
        .map(|h| CellData::new(h.trim()).bold())
        .collect();

    let mut rows: Vec<Vec<CellData>> = Vec::new();

    for line in lines {
        let mut row_metrics = Vec::new();
        let mut cells = Vec::new();

        for (i, val) in line.split(',').enumerate() {
            let trimmed = val.trim();

            if i > 0 {
                row_metrics.push(trimmed.parse::<f64>().unwrap_or(0.0));
            }

            cells.push(CellData::new(trimmed).center());
        }

        rows.push(cells);
    }

    (headers, rows)
}

#[component]
fn TableLoader(
    metric_type: ReadSignal<MetricType>,
    trigger_refresh: ReadSignal<usize>,
    cookie_name: Signal<String>,
) -> impl IntoView {
    let backend_csv = LocalResource::new(move || pull_from_backend(metric_type.get()));

    view! {
        <Suspense fallback=move || {
            view! { <p class="text-gray-400 p-4 font-mono">"Загрузка данных..."</p> }
        }>
            {move || {
                backend_csv
                    .get()
                    .map(|backend_data| {
                        let _ = trigger_refresh.get();
                        let (headers, rows) = parse_csv(&backend_data);
                        let (cookie_val, set_cookie_val) = use_cookie::<
                            String,
                            FromToStringCodec,
                        >(&cookie_name.get());
                        let raw_cookie = cookie_val.get().unwrap_or_default();
                        let user_rows: Vec<Vec<CellData>> = if raw_cookie.trim().is_empty() {
                            vec![]
                        } else {
                            let header_line = backend_data.lines().next().unwrap_or("").to_string();
                            let cookie_lines = raw_cookie.replace('|', "\n");
                            let mini_csv = format!("{}\n{}", header_line, cookie_lines.trim());
                            let (_, u_rows) = parse_csv(&mini_csv);
                            u_rows
                        };
                        let on_delete_user_row = Callback::new(move |idx: usize| {
                            let current = cookie_val.get().unwrap_or_default();
                            let remaining: Vec<&str> = current
                                .split('|')
                                .enumerate()
                                .filter_map(|(i, line)| (i != idx).then_some(line))
                                .collect();
                            if remaining.is_empty() {
                                set_cookie_val.set(None);
                            } else {
                                set_cookie_val.set(Some(remaining.join("|")));
                            }
                        });

                    view! {
                        <DenseTable
                            headers=headers
                            rows=rows
                            user_rows=user_rows
                            on_delete_user_row=on_delete_user_row
                            freeze_first_col=true
                            show_row_numbers=true
                        />
                    }
                })
            }}
        </Suspense>
    }
}