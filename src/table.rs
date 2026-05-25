use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use crate::sort_button::{Sort, SortButton};


#[derive(Clone, Debug, PartialEq)]
pub struct CellData {
    pub value: String,
    pub bold: bool,
    pub align: CellAlign,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl CellData {
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), bold: false, align: CellAlign::Left }
    }
    pub fn bold(mut self) -> Self { self.bold = true; self }
    pub fn center(mut self) -> Self { self.align = CellAlign::Center; self }
    pub fn right(mut self) -> Self { self.align = CellAlign::Right; self }
    pub fn left(mut self) -> Self { self.align = CellAlign::Left; self }
}


#[derive(Clone, Debug)]
#[derive(PartialEq)]
struct RowEntry {
    cells: Vec<CellData>,
    is_user: bool,
    user_idx: usize,
}


#[component]
pub fn DenseTable(
    mut headers: Vec<CellData>,
    rows: Vec<Vec<CellData>>,
    #[prop(default = vec![])] user_rows: Vec<Vec<CellData>>,
    on_delete_user_row: Callback<usize>,
    #[prop(default = false)] freeze_first_col: bool,
    #[prop(default = true)]  show_row_numbers: bool,
    #[prop(default = false)] freeze_headers: bool,
    #[prop(default = true)]  freeze_last_col: bool,
) -> impl IntoView {
    let (selected_cell, set_selected_cell) = signal::<Option<(usize, usize)>>(None);
    let (hovered_row, set_hovered_row) = signal::<Option<usize>>(None);
    let (normalized, set_normalized) = signal(false);

    headers.push(CellData::new("Оценка").bold());
    let col_count = headers.len();

    let metric_count = col_count - 2;
    let (weights, set_weights) = signal::<Vec<f64>>(vec![0.0; metric_count]);
    let (sort_col, set_sort_col) = signal::<Option<(usize, Sort)>>(None);

    let border_col  = "#4a5568";
    let header_bg   = "#1e2433";
    let header_fg   = "#e2e8f0";
    let base_borders = format!(
        "border-right: 3px solid {border_col}; border-bottom: 3px solid {border_col};"
    );

    let combined_entries: Vec<RowEntry> = rows
        .into_iter()
        .map(|cells| RowEntry { cells, is_user: false, user_idx: 0 })
        .chain(
            user_rows
                .into_iter()
                .enumerate()
                .map(|(i, cells)| RowEntry { cells, is_user: true, user_idx: i }),
        )
        .collect();

    let rows_with_results = Memo::new(move |_| {
        let current_weights = weights.get();
        let weight_sum      = current_weights.iter().sum::<f64>();
        let norm            = normalized.get();

        let mut entries = combined_entries.clone();

        entries.iter_mut().for_each(|entry| {
            let mut score: f64 = entry.cells
                .iter()
                .skip(1)
                .zip(current_weights.iter())
                .map(|(cell, w)| cell.value.parse::<f64>().unwrap_or(0.0) * w)
                .sum();

            if norm && weight_sum != 0.0 {
                score /= weight_sum;
            }

            entry.cells.push(CellData::new(format!("{:.3}", score)).bold());
        });

        if let Some((sort_idx, sort)) = sort_col.get() {
            entries.sort_by(|a, b| {
                let ca = a.cells.get(sort_idx + 1);
                let cb = b.cells.get(sort_idx + 1);
                if let (Some(ca), Some(cb)) = (ca, cb) {
                    let ord = match (ca.value.parse::<f64>(), cb.value.parse::<f64>()) {
                        (Ok(fa), Ok(fb)) =>
                            fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal),
                        _ => ca.value.cmp(&cb.value),
                    };
                    if sort == Sort::ASC { ord.reverse() } else { ord }
                } else {
                    std::cmp::Ordering::Equal
                }
            });
        }

        entries
    });

    view! {
        <div
            class="shadow-xl rounded select-none"
            style=format!(
                "background:#0f1219; padding:6px; \
                 border:3px solid {border_col}; border-radius:6px;",
            )
        >
            <div class="overflow-auto" style="width:100%;">
                <table
                    class="font-mono text-[13px]"
                    style=format!(
                        "border-collapse:separate; border-spacing:0; \
                         border-top:3px solid {border_col};",
                    )
                >
                    <thead>
                        <tr>
                            {show_row_numbers
                                .then(|| {
                                    let style = format!(
                                        "background:{header_bg}; {base_borders} \
                                     box-shadow: inset 3px 0 0 0 {border_col}; \
                                     position:sticky; left:0; z-index:60; \
                                     {}",
                                        if freeze_headers { "top:0;" } else { "" },
                                    );
                                    view! { <th style=style></th> }
                                })}
                            {headers
                                .iter()
                                .enumerate()
                                .map(|(col_idx, h)| {
                                    let is_last = col_idx == col_count - 1;
                                    let is_first = col_idx == 0;
                                    let pos_style = if is_last && freeze_last_col {
                                        format!(
                                            "position:sticky; right:0; z-index:65; \
                                         border-left:3px solid {border_col};",
                                        )
                                    } else if freeze_headers || (freeze_first_col && is_first) {
                                        "position:sticky; left:34px; z-index:55;".to_string()
                                    } else {
                                        "position:relative; z-index:40;".to_string()
                                    };
                                    let top_style = if freeze_headers { "top:0;" } else { "" };

                                    view! {
                                        <th
                                            class="whitespace-nowrap"
                                            style=format!(
                                                "background:{header_bg}; color:{header_fg}; \
                                             {base_borders} padding:10px 16px; \
                                             {top_style} {pos_style}",
                                            )
                                        >
                                            <div style="display:flex; flex-direction:column; \
                                            align-items:center; gap:4px;">
                                                <span>{h.value.clone()}</span>

                                                {(!is_last && !is_first && !freeze_headers)
                                                    .then(|| {
                                                        let w_idx = col_idx - 1;
                                                        view! {
                                                            <div style="display:flex; align-items:center; \
                                                            gap:6px; margin-top:2px;">
                                                                <input
                                                                    type="text"
                                                                    inputmode="decimal"
                                                                    step="0.05"
                                                                    min="0"
                                                                    max="1"
                                                                    prop:value=move || {
                                                                        let v = weights.get().get(w_idx).copied().unwrap_or(0.0);
                                                                        if v == 0.0 {
                                                                            String::new()
                                                                        } else {
                                                                            let s = format!("{:.2}", v);
                                                                            s.trim_end_matches('0').trim_end_matches('.').to_string()
                                                                        }
                                                                    }
                                                                    style="text-align:center;"
                                                                    class="w-16 bg-[#0f1219] border \
                                                                    border-[#4a5568] rounded \
                                                                    focus:border-[#4a9eff] \
                                                                    focus:outline-none"
                                                                    on:input=move |ev: web_sys::Event| {
                                                                        let raw = event_target_value(&ev);
                                                                        let mut has_dot = false;
                                                                        let filtered: String = raw
                                                                            .chars()
                                                                            .filter(|c| {
                                                                                if c.is_ascii_digit() {
                                                                                    true
                                                                                } else if *c == '.' && !has_dot {
                                                                                    has_dot = true;
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            })
                                                                            .collect();
                                                                        if filtered != raw {
                                                                            let el: web_sys::HtmlInputElement = ev
                                                                                .target()
                                                                                .unwrap()
                                                                                .unchecked_into();
                                                                            el.set_value(&filtered);
                                                                        }
                                                                        if filtered.is_empty() {
                                                                            set_weights
                                                                                .update(|w| {
                                                                                    if let Some(slot) = w.get_mut(w_idx) {
                                                                                        *slot = 0.0;
                                                                                    }
                                                                                });
                                                                        } else if let Ok(val) = filtered.parse::<f64>() {
                                                                            set_weights
                                                                                .update(|w| {
                                                                                    if let Some(slot) = w.get_mut(w_idx) {
                                                                                        *slot = val.clamp(0.0, 1.0);
                                                                                    }
                                                                                });
                                                                        }
                                                                    }
                                                                    on:change=move |ev: web_sys::Event| {
                                                                        let val = event_target_value(&ev)
                                                                            .parse::<f64>()
                                                                            .unwrap_or(0.0)
                                                                            .clamp(0.0, 1.0);
                                                                        set_weights
                                                                            .update(|w| {
                                                                                if let Some(slot) = w.get_mut(w_idx) {
                                                                                    *slot = val;
                                                                                }
                                                                            });
                                                                    }
                                                                />
                                                                <SortButton col_idx=w_idx set_sort_col sort_col />
                                                            </div>
                                                        }
                                                    })}

                                                {is_last
                                                    .then(|| {
                                                        view! {
                                                            <div style="display:flex; align-items:center; \
                                                            gap:6px; margin-top:2px;">
                                                                <button
                                                                    class=move || {
                                                                        if normalized.get() {
                                                                            "w-16 bg-[#1a2030] border \
                                                             border-[#4a9eff] rounded \
                                                             text-center focus:outline-none \
                                                             text-[#4a9eff]"
                                                                        } else {
                                                                            "w-16 bg-[#0f1219] border \
                                                             border-[#4a5568] rounded \
                                                             text-center focus:outline-none \
                                                             text-white"
                                                                        }
                                                                    }
                                                                    on:click=move |_| { set_normalized.update(|v| *v = !*v) }
                                                                >
                                                                    "Норм"
                                                                </button>
                                                                <SortButton col_idx=col_idx - 1 set_sort_col sort_col />
                                                            </div>
                                                        }
                                                    })}
                                            </div>
                                        </th>
                                    }
                                })
                                .collect_view()}
                        </tr>
                    </thead>

                    <tbody>
                        {move || {
                            rows_with_results
                                .get()
                                .into_iter()
                                .enumerate()
                                .map(|(row_idx, entry)| {
                                    let base_borders = base_borders.clone();
                                    let is_user = entry.is_user;
                                    let user_idx = entry.user_idx;

                                    view! {
                                        <tr
                                            on:mouseenter=move |_| set_hovered_row.set(Some(row_idx))
                                            on:mouseleave=move |_| set_hovered_row.set(None)
                                        >
                                            {show_row_numbers
                                                .then(|| {
                                                    let base_borders = base_borders.clone();
                                                    view! {
                                                        <td style=format!(
                                                            "color:#4a6080; background:#161c2b; \
                                                         {base_borders} \
                                                         box-shadow: inset 3px 0 0 0 {border_col}; \
                                                         position:sticky; left:0; z-index:20; \
                                                         padding:10px 12px;",
                                                        )>{row_idx + 1}</td>
                                                    }
                                                })}

                                            {entry
                                                .cells
                                                .into_iter()
                                                .enumerate()
                                                .map(|(col_idx, cell)| {
                                                    let is_last_col = col_idx == col_count - 1;
                                                    let is_first_col = col_idx == 0;
                                                    let sticky_style = if is_last_col && freeze_last_col {
                                                        format!(
                                                            "position:sticky; right:0; z-index:20; \
                                                         border-left:3px solid {border_col};",
                                                        )
                                                    } else if freeze_first_col && is_first_col {
                                                        let left = if show_row_numbers {
                                                            "left:35px"
                                                        } else {
                                                            "left:0"
                                                        };
                                                        format!(
                                                            "position:sticky; {left}; z-index:20; \
                                                         border-right:3px solid {border_col};",
                                                        )
                                                    } else {
                                                        String::new()
                                                    };
                                                    let bg = move || {
                                                        if selected_cell.get() == Some((row_idx, col_idx)) {
                                                            "#0d2a4a"
                                                        } else if hovered_row.get() == Some(row_idx) {
                                                            "#2a3550"
                                                        } else {
                                                            "#161c2b"
                                                        }
                                                    };
                                                    let show_delete = is_user && is_first_col;
                                                    let base_borders = base_borders.clone();

                                                    view! {
                                                        <td
                                                            class="whitespace-nowrap transition-colors"
                                                            style=move || {
                                                                format!(
                                                                    "color:#c8d8f0; padding:10px 16px; \
                                                             background:{}; {base_borders} \
                                                             {sticky_style}",
                                                                    bg(),
                                                                )
                                                            }
                                                            on:click=move |_| {
                                                                set_selected_cell.set(Some((row_idx, col_idx)))
                                                            }
                                                        >
                                                            {if show_delete {
                                                                view! {
                                                                    <div style="display:flex; \
                                                                    align-items:center; \
                                                                    gap:6px;">
                                                                        <span>{cell.value}</span>
                                                                        <button
                                                                            title="Удалить"
                                                                            style="flex-shrink:0; \
                                                                            line-height:1; \
                                                                            background:transparent; \
                                                                            border:none; \
                                                                            color:#e05c5c; \
                                                                            cursor:pointer; \
                                                                            font-size:14px; \
                                                                            padding:0 2px;"
                                                                            on:click=move |ev| {
                                                                                ev.stop_propagation();
                                                                                on_delete_user_row.run(user_idx);
                                                                            }
                                                                        >
                                                                            "✕"
                                                                        </button>
                                                                    </div>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! { <span>{cell.value}</span> }.into_any()
                                                            }}
                                                        </td>
                                                    }
                                                })
                                                .collect_view()}
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </div>

            <span class="font-mono text-[12px] block">
                "Введите веса и, при необходимости, нормализуйте оценки по кнопке «Норм» в колонке оценок."
            </span>
            <span class="font-mono text-[12px] block">
                "Отсортируйте по требуемым значениям."
            </span>
        </div>
    }
}