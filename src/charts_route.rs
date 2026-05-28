use crate::MetricType;
use crate::pull_from_backend;
use std::collections::BTreeMap;

use leptos::component;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use crate::bench_hist::{parse_benchmark_csv, BenchmarkHistogram};
use crate::chart::{AxisConfig, Chart, ChartPoint, ChartSeries, NodeId, PointShape};

#[derive(Debug, Clone)]
struct PrRow {
    model:     String,
    threshold: f64,
    precision: f64,
    recall:    f64,
    f1:        f64,
}

fn parse_csv(raw: &str) -> Vec<PrRow> {
    let text = raw.trim_start_matches('\u{feff}');

    let sep = detect_separator(text);
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());

    let header_line = match lines.next() {
        Some(h) => h,
        None => return vec![],
    };

    let headers: Vec<String> = header_line
        .split(sep)
        .map(|h| h.trim().to_string())
        .collect();

    let model_col = headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case("model"));

    if let Some(model_idx) = model_col {
        parse_flat(&headers, model_idx, sep, lines)
    } else {
        parse_wide(&headers, sep, lines)
    }
}

pub(crate) fn detect_separator(text: &str) -> char {
    let first = text.lines().next().unwrap_or("");
    let semis  = first.chars().filter(|&c| c == ';').count();
    let commas = first.chars().filter(|&c| c == ',').count();
    if semis >= commas { ';' } else { ',' }
}

fn parse_flat<'a>(
    headers: &[String],
    model_idx: usize,
    sep: char,
    lines: impl Iterator<Item = &'a str>,
) -> Vec<PrRow> {
    let col = |name: &str| {
        headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(name))
    };

    let threshold_idx = col("threshold").unwrap_or(1);
    let precision_idx = col("precision").unwrap_or(2);
    let recall_idx    = col("recall").unwrap_or(3);
    let f1_idx        = col("f1_score")
        .or_else(|| col("f1"))
        .unwrap_or(4);

    let mut rows = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(sep).collect();
        if fields.len() <= f1_idx { continue; }

        let model     = fields[model_idx].trim().to_string();
        let threshold = fields[threshold_idx].trim().parse().unwrap_or(f64::NAN);
        let precision = fields[precision_idx].trim().parse().unwrap_or(f64::NAN);
        let recall    = fields[recall_idx].trim().parse().unwrap_or(f64::NAN);
        let f1        = fields[f1_idx].trim().parse().unwrap_or(f64::NAN);

        if model.is_empty() || threshold.is_nan() { continue; }

        rows.push(PrRow { model, threshold, precision, recall, f1 });
    }
    rows
}

fn parse_wide<'a>(
    headers: &[String],
    sep: char,
    lines: impl Iterator<Item = &'a str>,
) -> Vec<PrRow> {
    let threshold_idx = headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case("threshold"))
        .unwrap_or(0);

    let mut model_cols: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();

    for (i, h) in headers.iter().enumerate() {
        if i == threshold_idx { continue; }

        let (model, metric) = if let Some(pos) = h.rfind('_') {
            (&h[..pos], &h[pos + 1..])
        } else {
            continue;
        };

        let entry = model_cols
            .entry(model.to_string())
            .or_insert((usize::MAX, usize::MAX, usize::MAX));

        if metric.eq_ignore_ascii_case("precision") {
            entry.0 = i;
        } else if metric.eq_ignore_ascii_case("recall") {
            entry.1 = i;
        } else if metric.eq_ignore_ascii_case("f1") || metric.eq_ignore_ascii_case("f1_score") {
            entry.2 = i;
        }
    }

    let mut rows = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(sep).collect();
        let threshold: f64 = fields
            .get(threshold_idx)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(f64::NAN);
        if threshold.is_nan() { continue; }

        for (model, (pi, ri, fi)) in &model_cols {
            let get = |idx: usize| -> f64 {
                fields.get(idx).and_then(|v| v.trim().parse().ok()).unwrap_or(f64::NAN)
            };
            rows.push(PrRow {
                model:     model.clone(),
                threshold,
                precision: get(*pi),
                recall:    get(*ri),
                f1:        get(*fi),
            });
        }
    }
    rows
}


fn rows_to_series(rows: &[PrRow]) -> Vec<ChartSeries> {
    let mut precision = Vec::with_capacity(rows.len());
    let mut recall    = Vec::with_capacity(rows.len());
    let mut f1        = Vec::with_capacity(rows.len());

    for r in rows {
        let pt = |y: f64| ChartPoint { x: r.threshold, y, label: None };
        precision.push(pt(r.precision));
        recall.push(pt(r.recall));
        f1.push(pt(r.f1));
    }

    vec![
        ChartSeries {
            name:      "Precision".to_string(),
            color:     "#3b82f6".to_string(),
            shape:     PointShape::Circle,
            connected: true,
            points:    precision,
        },
        ChartSeries {
            name:      "Recall".to_string(),
            color:     "#ef4444".to_string(),
            shape:     PointShape::Circle,
            connected: true,
            points:    recall,
        },
        ChartSeries {
            name:      "F1 Score".to_string(),
            color:     "#000000".to_string(),
            shape:     PointShape::Circle,
            connected: true,
            points:    f1,
        },
    ]
}

#[component]
pub fn ChartsRoute(metric_type: ReadSignal<MetricType>) -> impl IntoView {
    let csv_resource = LocalResource::new(move || {
        let current_type = metric_type.get().to_ml();
        async move {
            pull_from_backend(current_type).await.unwrap_or_else(|e| {
                tracing::warn!("Fetch failed, cycle continues: {e}");
                String::new()
            })
        }
    });

    let x_axis = AxisConfig {
        label: "Threshold".to_string(),
        log: false,
        min: 0.0,
        max: 1.0,
        ticks: None,
    };
    let y_axis = AxisConfig {
        label: "Score".to_string(),
        log: false,
        min: 0.0,
        max: 1.0,
        ticks: None,
    };

    let (sync_x, set_sync_x) = signal(None::<f64>);
    let (selected, set_selected) = signal(None::<NodeId>);

    view! {
        {move || {
            match csv_resource.get() {
                None => {
                    view! {
                        <div style="padding:48px; color:#94a3b8; font-family:'JetBrains Mono',monospace;">
                            "Loading…"
                        </div>
                    }
                        .into_any()
                }
                Some(raw) if raw.trim().is_empty() => {

                    view! {
                        <div style="padding:48px; color:#94a3b8; font-family:'JetBrains Mono',monospace;">
                            "No data available"
                        </div>
                    }
                        .into_any()
                }
                Some(raw) => {
                    if matches!(metric_type.get(), MetricType::LLM) {
                        let benchmarks = parse_benchmark_csv(&raw);
                        if benchmarks.is_empty() {
                            return 

                            view! {
                                <div style="padding:48px; color:#94a3b8; font-family:'JetBrains Mono',monospace;">
                                    "No benchmark data found"
                                </div>
                            }
                                .into_any();
                        }
                        let histograms: Vec<_> = benchmarks
                            .into_iter()
                            .map(|row| {

                                view! { <BenchmarkHistogram row=row /> }
                            })
                            .collect();

                        view! {
                            <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(340px, 1fr)); gap:20px; padding:24px;">
                                {histograms}
                            </div>
                        }
                            .into_any()
                    } else {
                        let all_rows = parse_csv(&raw);
                        let mut by_model: BTreeMap<String, Vec<PrRow>> = BTreeMap::new();
                        for row in all_rows {
                            by_model.entry(row.model.clone()).or_default().push(row);
                        }
                        for rows in by_model.values_mut() {
                            rows.sort_by(|a, b| a.threshold.partial_cmp(&b.threshold).unwrap());
                        }
                        let charts: Vec<_> = by_model
                            .into_iter()
                            .map(|(model_name, rows)| {
                                let series = rows_to_series(&rows);
                                let label = model_name.clone();
                                let xa = x_axis.clone();
                                let ya = y_axis.clone();

                                view! {
                                    <div
                                        style="
                                        background: #151c2c;
                                        border: 1px solid #1e3a6e;
                                        border-radius: 8px;
                                        padding: 20px;
                                        transition: border-color 0.2s ease, box-shadow 0.2s ease;
                                        "
                                        on:mouseenter=move |e| {
                                            let el = e
                                                .target()
                                                .unwrap()
                                                .unchecked_into::<web_sys::HtmlElement>();
                                            let _ = el.style().set_property("border-color", "#4a9eff");
                                            let _ = el
                                                .style()
                                                .set_property(
                                                    "box-shadow",
                                                    "0 0 12px rgba(74,158,255,0.15)",
                                                );
                                        }
                                        on:mouseleave=move |e| {
                                            let el = e
                                                .target()
                                                .unwrap()
                                                .unchecked_into::<web_sys::HtmlElement>();
                                            let _ = el.style().set_property("border-color", "#1e3a6e");
                                            let _ = el.style().set_property("box-shadow", "none");
                                        }
                                    >
                                        <div style="font-family: 'JetBrains Mono', monospace; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; color: #94a3b8; margin-bottom: 4px;">
                                            "MODEL"
                                        </div>
                                        <div style="font-family: 'JetBrains Mono', monospace; font-size: 14px; font-weight: 700; color: #4a9eff; margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid #1e3a6e;">
                                            {label}
                                        </div>
                                        <Chart
                                            series=series
                                            x_axis=xa
                                            y_axis=ya
                                            selected=selected
                                            set_selected=set_selected
                                            sync_x=sync_x
                                            set_sync_x=set_sync_x
                                            width=640.0
                                            height=420.0
                                        />
                                    </div>
                                }
                            })
                            .collect();

                        view! {
                            <div style="display:grid; grid-template-columns:1fr 1fr; gap:24px; padding:24px;">
                                {charts}
                            </div>
                        }
                            .into_any()
                    }
                }
            }
        }}
    }
}