mod table;
mod backend;
mod metric_list;
mod agent;
mod sort_button;
mod add_metrics;
mod chart;
mod charts_route;
mod metric_page;
mod navbar;
mod bench_hist;

use crate::add_metrics::AddMetrics;
use crate::backend::{pull_from_backend, MetricType};
use crate::charts_route::ChartsRoute;
use crate::metric_list::MetricList;
use crate::metric_page::MetricRoute;
use leptos::prelude::*;
use leptos::server::codee::string::FromToStringCodec;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;
use leptos_use::use_cookie;
use table::{CellData, DenseTable};
use crate::navbar::Navbar;

#[component]
fn RouteSyncer(set_active_route: WriteSignal<String>) -> impl IntoView {
    let loc = use_location();
    Effect::new(move |_| {
        set_active_route.set(loc.pathname.get());
    });
    view! {}
}

fn main() {
    console_error_panic_hook::set_once();

    let (metric_type, set_metric_type) = signal::<MetricType>(MetricType::LLM);
    let (cookie_refresh, set_cookie_refresh) = signal(0usize);

    let (active_route, set_active_route) = signal(String::from("/"));

    mount_to(
        document().get_element_by_id("app-mount").unwrap().unchecked_into(),
        {
            let set_active_route = set_active_route;
            move || view! {
                <Router>
                    <RouteSyncer set_active_route />
                    <Routes fallback=|| "Not found.">
                        <Route
                            path=path!("")
                            view=move || {
                                view! { <MetricRoute metric_type trigger_refresh=cookie_refresh /> }
                            }
                        />
                        <Route
                            path=path!("/ml")
                            view=move || view! { <ChartsRoute metric_type /> }
                        />
                    </Routes>
                </Router>
            }
        },
    ).forget();

    mount_to(
        document().get_element_by_id("nav-mount").unwrap().unchecked_into(),
        move || view! { <Navbar /> },
    ).forget();

    mount_to(
        document().get_element_by_id("add-mount").unwrap().unchecked_into(),
        {
            let active_route = active_route;
            move || {
                let is_ml_route = Memo::new(move |_| active_route.get() == "/ml");

                view! {
                    <Show when=move || !is_ml_route.get()>
                        <AddMetrics set_trigger_refresh=set_cookie_refresh />
                    </Show>
                }
            }
        },
    ).forget();

    mount_to(
        document().get_element_by_id("list-mount").unwrap().unchecked_into(),
        move || view! { <MetricList metric_type set_metric_type /> },
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
    let backend_csv = LocalResource::new(move || {
        let current_type = metric_type.get();

        async move {
            pull_from_backend(current_type).await.unwrap_or_else(|e| {
                tracing::warn!("Fetch failed, cycle continues: {e}");
                String::new()
            })
        }
    });

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