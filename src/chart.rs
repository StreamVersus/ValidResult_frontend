#![allow(unused)]
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

impl ChartPoint {
    pub fn xy(x: f64, y: f64) -> Self {
        Self { x, y, label: None }
    }
    pub fn with_label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointShape {
    Circle,
    Square,
    Diamond,
}

#[derive(Clone, Debug)]
pub struct ChartSeries {
    pub name: String,
    pub color: String,
    pub shape: PointShape,
    pub connected: bool,
    pub points: Vec<ChartPoint>,
}

impl ChartSeries {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
            shape: PointShape::Circle,
            connected: true,
            points: vec![],
        }
    }
    pub fn shape(mut self, s: PointShape) -> Self {
        self.shape = s;
        self
    }
    pub fn connected(mut self, c: bool) -> Self {
        self.connected = c;
        self
    }
    pub fn points(mut self, pts: Vec<ChartPoint>) -> Self {
        self.points = pts;
        self
    }
}

#[derive(Clone, Debug)]
pub struct AxisConfig {
    pub label: String,
    pub log: bool,
    pub min: f64,
    pub max: f64,
    pub ticks: Option<Vec<f64>>,
}

impl AxisConfig {
    pub fn linear(label: impl Into<String>, min: f64, max: f64) -> Self {
        Self { label: label.into(), log: false, min, max, ticks: None }
    }
    pub fn log(label: impl Into<String>, min: f64, max: f64) -> Self {
        Self { label: label.into(), log: true, min, max, ticks: None }
    }
    pub fn with_ticks(mut self, t: Vec<f64>) -> Self {
        self.ticks = Some(t);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeId {
    pub series: usize,
    pub point: usize,
}

fn to_px(val: f64, cfg: &AxisConfig, out_min: f64, out_max: f64) -> f64 {
    let t = if cfg.log {
        let lv   = val.max(1e-12).log10();
        let lmin = cfg.min.max(1e-12).log10();
        let lmax = cfg.max.max(1e-12).log10();
        if (lmax - lmin).abs() < 1e-12 { 0.0 } else { (lv - lmin) / (lmax - lmin) }
    } else {
        let range = cfg.max - cfg.min;
        if range.abs() < 1e-12 { 0.0 } else { (val - cfg.min) / range }
    };
    out_min + t.clamp(0.0, 1.0) * (out_max - out_min)
}

fn gen_ticks(cfg: &AxisConfig, target: usize) -> Vec<f64> {
    if let Some(ref custom) = cfg.ticks {
        return custom.clone();
    }
    if cfg.log {
        let lo = cfg.min.max(1e-12).log10().floor() as i32;
        let hi = cfg.max.max(1e-12).log10().ceil()  as i32;
        let mut ticks: Vec<f64> = vec![];
        for exp in (lo - 1)..=(hi + 1) {
            for &mantissa in &[1.0_f64, 2.0, 3.0, 5.0] {
                let v = mantissa * 10f64.powi(exp);
                if v >= cfg.min * 0.98 && v <= cfg.max * 1.02 {
                    ticks.push(v);
                }
            }
        }
        ticks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ticks.dedup_by(|a, b| (*a - *b).abs() < *b * 1e-9);
        ticks
    } else {
        let range = cfg.max - cfg.min;
        let raw   = range / target as f64;
        let mag   = 10f64.powf(raw.log10().floor());
        let step  = [1.0_f64, 2.0, 5.0, 10.0]
            .iter().copied()
            .map(|f| f * mag)
            .find(|&s| s >= raw)
            .unwrap_or(raw);
        let first = (cfg.min / step).ceil() * step;
        let mut ticks = vec![];
        let mut v = first;
        while v <= cfg.max + step * 0.01 {
            ticks.push(v);
            v += step;
        }
        ticks
    }
}

fn fmt_num(v: f64) -> String {
    if v == 0.0 { return "0".into(); }
    let av = v.abs();
    if av >= 10_000.0      { format!("{:.0}", v) }
    else if av >= 1_000.0  { format!("{:.0}", v) }
    else if av >= 100.0    { format!("{:.0}", v) }
    else if av >= 10.0     { format!("{:.1}", v) }
    else if av >= 1.0      { format!("{:.2}", v) }
    else                   { format!("{:.3}", v) }
}

#[derive(Clone, Debug)]
struct TipState {
    px: f64,
    py: f64,
    vx: f64,
    vy: f64,
    name: String,
    color: String,
    label: Option<String>,
}

fn render_shape(shape: PointShape, cx: f64, cy: f64, r: f64, color: String) -> AnyView {
    match shape {
        PointShape::Circle => view! { <circle cx=cx cy=cy r=r class="chart-point-shape" fill=color /> }.into_any(),

        PointShape::Square => view! {
            <rect
                x=cx - r
                y=cy - r
                width=r * 2.0
                height=r * 2.0
                class="chart-point-shape"
                fill=color
            />
        }.into_any(),

        PointShape::Diamond => {
            let pts = format!(
                "{cx},{top} {right},{cy} {cx},{bot} {left},{cy}",
                top   = cy - r * 1.35,
                right = cx + r * 1.1,
                bot   = cy + r * 1.35,
                left  = cx - r * 1.1,
            );
            view! { <polygon points=pts class="chart-point-shape" fill=color /> }.into_any()
        }
    }
}

#[component]
pub fn Chart(
    series: Vec<ChartSeries>,
    x_axis: AxisConfig,
    y_axis: AxisConfig,
    selected: ReadSignal<Option<NodeId>>,
    set_selected: WriteSignal<Option<NodeId>>,
    #[prop(optional)] sync_x: Option<ReadSignal<Option<f64>>>,
    #[prop(optional)] set_sync_x: Option<WriteSignal<Option<f64>>>,
    #[prop(default = 560.0_f64)] width: f64,
    #[prop(default = 400.0_f64)] height: f64,
) -> impl IntoView {
    let ml = 64.0_f64;
    let mr = 20.0_f64;
    let mt = 28.0_f64;
    let mb = 88.0_f64;
    let iw = width  - ml - mr;
    let ih = height - mt - mb;

    let x_cfg_sv = StoredValue::new_local(x_axis.clone());
    let y_cfg_sv = StoredValue::new_local(y_axis.clone());

    let sx = move |v: f64| to_px(v, &x_cfg_sv.get_value(), ml, ml + iw);
    let sy = move |v: f64| to_px(v, &y_cfg_sv.get_value(), mt + ih, mt);

    let px_to_x = move |svg_px: f64| {
        let cfg = x_cfg_sv.get_value();
        let t = (svg_px - ml) / iw;
        cfg.min + t * (cfg.max - cfg.min)
    };

    let series_px: Vec<Vec<(f64, f64)>> = series
        .iter()
        .map(|s| s.points.iter().map(|p| (sx(p.x), sy(p.y))).collect())
        .collect();

    let x_ticks = gen_ticks(&x_axis, 6);
    let y_ticks = gen_ticks(&y_axis, 6);

    let series_sv = StoredValue::new_local(series.clone());
    let series_px_sv = StoredValue::new_local(series_px.clone());
    let x_label = x_axis.label.clone();
    let y_label = y_axis.label.clone();

    let tip: RwSignal<Option<TipState>> = RwSignal::new(None);

    view! {
        <div
            class="chart-container"
            style=format!("width:{w}px; height:{h}px;", w = width, h = height)
        >
            <svg
                width=width
                height=height
                class="chart-svg"
                on:mousemove=move |e| {
                    if let Some(setter) = set_sync_x {
                        let bounding = e
                            .current_target()
                            .unwrap()
                            .unchecked_into::<web_sys::Element>()
                            .get_bounding_client_rect();
                        let svg_x = e.client_x() as f64 - bounding.left();
                        if svg_x >= ml && svg_x <= ml + iw {
                            setter.set(Some(px_to_x(svg_x)));
                        } else {
                            setter.set(None);
                        }
                    }
                }
                on:mouseleave=move |_| {
                    tip.set(None);
                    if let Some(s) = set_sync_x {
                        s.set(None);
                    }
                }
            >
                <rect x=ml y=mt width=iw height=ih class="chart-bg" rx="2" />

                {x_ticks
                    .iter()
                    .map(|&v| {
                        let px = sx(v);
                        view! { <line x1=px x2=px y1=mt y2=mt + ih class="chart-grid-line" /> }
                    })
                    .collect_view()}

                {y_ticks
                    .iter()
                    .map(|&v| {
                        let py = sy(v);
                        view! { <line x1=ml x2=ml + iw y1=py y2=py class="chart-grid-line" /> }
                    })
                    .collect_view()}

                <rect x=ml y=mt width=iw height=ih class="chart-plot-border" />

                {x_ticks
                    .iter()
                    .map(|&v| {
                        let px = sx(v);
                        let lbl = fmt_num(v);
                        view! {
                            <line x1=px x2=px y1=mt + ih y2=mt + ih + 5.0 class="chart-axis-tick" />
                            <text x=px y=mt + ih + 16.0 class="chart-axis-label">
                                {lbl}
                            </text>
                        }
                    })
                    .collect_view()}

                {y_ticks
                    .iter()
                    .map(|&v| {
                        let py = sy(v);
                        let lbl = fmt_num(v);
                        view! {
                            <line x1=ml - 5.0 x2=ml y1=py y2=py class="chart-axis-tick" />
                            <text x=ml - 8.0 y=py + 3.5 class="chart-axis-label chart-axis-label-y">
                                {lbl}
                            </text>
                        }
                    })
                    .collect_view()}

                <text x=ml + iw / 2.0 y=height - 52.0 class="chart-axis-title">
                    {x_label.clone()}
                </text>
                <text
                    x=13.0
                    y=mt + ih / 2.0
                    class="chart-axis-title"
                    transform=format!("rotate(-90,13,{})", mt + ih / 2.0)
                >
                    {y_label.clone()}
                </text>

                {series_sv
                    .get_value()
                    .iter()
                    .zip(series_px_sv.get_value().iter())
                    .filter_map(|(s, px_vec)| {
                        if !s.connected || px_vec.len() < 2 {
                            return None;
                        }
                        let pts: String = px_vec
                            .iter()
                            .map(|(px, py)| format!("{px:.1},{py:.1}"))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let color = s.color.clone();
                        Some(
                            view! {
                                <polyline
                                    points=pts
                                    class="chart-series-line"
                                    style=format!("stroke:{color};")
                                />
                            },
                        )
                    })
                    .collect_view()}

                {series_sv
                    .get_value()
                    .into_iter()
                    .zip(series_px_sv.get_value().into_iter())
                    .enumerate()
                    .flat_map(|(si, (s, px_vec))| {
                        px_vec
                            .into_iter()
                            .zip(s.points.into_iter())
                            .enumerate()
                            .map(move |(pi, ((px, py), pt))| {
                                let color_glow = s.color.clone();
                                let color_shape = s.color.clone();
                                let color_tip = s.color.clone();
                                let sname = s.name.clone();
                                let shape = s.shape;
                                let vx = pt.x;
                                let vy = pt.y;
                                let tip_label = pt.label.clone();
                                let nid_sel = NodeId { series: si, point: pi };
                                let nid_click = NodeId { series: si, point: pi };
                                let is_sel = move || selected.get().as_ref() == Some(&nid_sel);

                                view! {
                                    <g
                                        class="chart-point-group"
                                        on:mouseenter=move |_| {
                                            tip.set(
                                                Some(TipState {
                                                    px,
                                                    py,
                                                    vx,
                                                    vy,
                                                    name: sname.clone(),
                                                    color: color_tip.clone(),
                                                    label: tip_label.clone(),
                                                }),
                                            )
                                        }
                                        on:mouseleave=move |_| tip.set(None)
                                        on:click=move |_| {
                                            set_selected
                                                .update(|cur| {
                                                    *cur = if cur.as_ref() == Some(&nid_click) {
                                                        None
                                                    } else {
                                                        Some(nid_click.clone())
                                                    };
                                                })
                                        }
                                    >
                                        {
                                            let value = is_sel.clone();
                                            move || {
                                                value
                                                    .clone()()
                                                    .then(|| {
                                                        view! {
                                                            <circle
                                                                cx=px
                                                                cy=py
                                                                r=16.0
                                                                class="chart-glow-halo"
                                                                style=format!("fill:{};", color_glow.clone())
                                                            />
                                                            <circle
                                                                cx=px
                                                                cy=py
                                                                r=11.5
                                                                class="chart-glow-ring"
                                                                style=format!("stroke:{};", color_glow.clone())
                                                            />
                                                        }
                                                    })
                                            }
                                        }
                                        {move || {
                                            let r = if is_sel() { 8.0 } else { 5.5 };
                                            render_shape(shape, px, py, r, color_shape.clone())
                                        }}
                                    </g>
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect_view()}

                {series_sv
                    .get_value()
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let lx = ml + 10.0 + i as f64 * 145.0;
                        let ly = height - 36.0;
                        let color = s.color.clone();
                        let name = s.name.clone();
                        let shape = s.shape;
                        view! {
                            <g>
                                {render_shape(shape, lx, ly, 5.0, color)}
                                <text x=lx + 11.0 y=ly + 4.0 class="chart-legend-text">
                                    {name}
                                </text>
                            </g>
                        }
                    })
                    .collect_view()}

                {move || {
                    let x_val = sync_x.and_then(|sig| sig.get())?;
                    let px = sx(x_val);
                    (px >= ml && px <= ml + iw)
                        .then(|| {
                            view! {
                                <line
                                    x1=px
                                    x2=px
                                    y1=mt
                                    y2=mt + ih
                                    class="chart-sync-line"
                                    style="pointer-events:none;"
                                />
                            }
                        })
                }}

            </svg>

            {
                let x_lbl = x_label.clone();
                let y_lbl = y_label.clone();
                move || {
                    tip.get()
                        .map(|t| {
                            let left = if t.px + 165.0 > width {
                                t.px - 152.0
                            } else {
                                t.px + 14.0
                            };
                            let top = if t.py + 100.0 > height { t.py - 92.0 } else { t.py - 14.0 };

                            view! {
                                <div
                                    class="chart-tooltip"
                                    style=format!(
                                        "left:{left:.0}px; top:{top:.0}px; border-left:3px solid {color};",
                                        color = t.color,
                                    )
                                >
                                    <div class="chart-tooltip-header">{t.name.clone()}</div>
                                    <div class="chart-tooltip-row">
                                        <span class="chart-tooltip-key">{x_lbl.clone()}</span>
                                        <span class="chart-tooltip-value">{fmt_num(t.vx)}</span>
                                    </div>
                                    <div class="chart-tooltip-row">
                                        <span class="chart-tooltip-key">{y_lbl.clone()}</span>
                                        <span class="chart-tooltip-value">{fmt_num(t.vy)}</span>
                                    </div>
                                    {t
                                        .label
                                        .map(|l| {
                                            view! { <div class="chart-tooltip-extra">{l}</div> }
                                        })}
                                </div>
                            }
                        })
                }
            }
        </div>
    }
}