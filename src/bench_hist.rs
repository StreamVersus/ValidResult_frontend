use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct BenchmarkRow {
    model: String,
    scores: [Option<f64>; 5], // [MMLU_Pro, GPQA, SWE-bench, LiveCodeBench, AIME]
}

const BENCHMARK_NAMES: [&str; 5] = [
    "MMLU-Pro",
    "GPQA Diamond",
    "SWE-bench Verified",
    "LiveCodeBench",
    "AIME 2025",
];

const BENCHMARK_COLORS: [&str; 5] = [
    "#3b82f6", // blue
    "#8b5cf6", // purple
    "#10b981", // emerald
    "#f59e0b", // amber
    "#ef4444", // red
];

pub fn parse_benchmark_csv(raw: &str) -> Vec<BenchmarkRow> {
    let text = raw.trim_start_matches('\u{feff}');
    let sep = crate::charts_route::detect_separator(text);
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());

    let headers: Vec<String> = match lines.next() {
        Some(h) => h.split(sep).map(|s| s.trim().to_string()).collect(),
        None => return vec![],
    };

    let benchmark_cols: [Option<usize>; 5] = [
        headers.iter().position(|h| h.eq_ignore_ascii_case("MMLU_Pro") || h.eq_ignore_ascii_case("MMLU-Pro")),
        headers.iter().position(|h| h.eq_ignore_ascii_case("GPQA_Diamond") || h.eq_ignore_ascii_case("GPQA Diamond")),
        headers.iter().position(|h| h.eq_ignore_ascii_case("SWE_bench_Verified") || h.eq_ignore_ascii_case("SWE-bench Verified")),
        headers.iter().position(|h| h.eq_ignore_ascii_case("LiveCodeBench")),
        headers.iter().position(|h| h.eq_ignore_ascii_case("AIME_2025") || h.eq_ignore_ascii_case("AIME 2025")),
    ];

    let model_col = headers.iter().position(|h| h.eq_ignore_ascii_case("model") || h.eq_ignore_ascii_case("model_id"));

    let mut rows = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(sep).collect();
        let model = model_col.and_then(|i| fields.get(i)).map(|s| s.trim().to_string()).unwrap_or_default();
        if model.is_empty() { continue; }

        let scores: [Option<f64>; 5] = benchmark_cols.map(|col_idx| {
            col_idx.and_then(|i| fields.get(i))
                .and_then(|v| {
                    let trimmed = v.trim();
                    if trimmed.eq_ignore_ascii_case("N/A") || trimmed.is_empty() {
                        None
                    } else {
                        trimmed.parse::<f64>().ok()
                    }
                })
        });

        rows.push(BenchmarkRow { model, scores });
    }
    rows
}

#[component]
pub fn BenchmarkHistogram(row: BenchmarkRow) -> impl IntoView {
    view! {
        <div style="
        background: #151c2c;
        border: 1px solid #1e3a6e;
        border-radius: 8px;
        padding: 16px 20px;
        min-width: 320px;
        ">
            <div style="
            font-family: 'JetBrains Mono', monospace;
            font-size: 13px;
            font-weight: 700;
            color: #4a9eff;
            margin-bottom: 12px;
            padding-bottom: 10px;
            border-bottom: 1px solid #1e3a6e;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            ">{row.model.clone()}</div>

            <div style="display: flex; flex-direction: column; gap: 8px;">
                {row
                    .scores
                    .iter()
                    .enumerate()
                    .map(|(i, &score_opt)| {
                        let name = BENCHMARK_NAMES[i];
                        let color = BENCHMARK_COLORS[i];
                        let value = score_opt.unwrap_or(-1.0);
                        let width_pct = if value >= 0.0 { value } else { 0.0 };
                        let score_color = if value >= 0.0 { "#e2e8f0" } else { "#64748b" };
                        let score_text = if value >= 0.0 {
                            format!("{:.1}", value)
                        } else {
                            "N/A".to_string()
                        };

                        view! {
                            <div style="display: flex; align-items: center; gap: 10px; height: 28px;">
                                <div style="
                                font-family: 'JetBrains Mono', monospace;
                                font-size: 11px;
                                color: #94a3b8;
                                width: 140px;
                                flex-shrink: 0;
                                white-space: nowrap;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                ">{name}</div>

                                <div style="
                                flex: 1;
                                height: 20px;
                                background: #0f172a;
                                border-radius: 4px;
                                position: relative;
                                overflow: hidden;
                                ">
                                    {if value >= 0.0 {
                                        view! {
                                            <div style=format!(
                                                "position:absolute; left:0; top:0; height:100%; width:{}%; background:{}; border-radius:4px; transition:width 0.2s ease;",
                                                width_pct,
                                                color,
                                            ) />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <div style="
                                            position:absolute; left:0; top:0; height:100%; width:100%;
                                            background: repeating-linear-gradient(
                                            45deg,
                                            #1e293b 0px,
                                            #1e293b 8px,
                                            #0f172a 8px,
                                            #0f172a 16px
                                            );
                                            border-radius:4px;
                                            " />
                                        }
                                            .into_any()
                                    }}
                                    <div style=format!(
                                        "position:absolute; right:8px; top:50%; transform:translateY(-50%);
                                    font-family: 'JetBrains Mono', monospace;
                                    font-size: 10px;
                                    font-weight: 600;
                                    color: {};",
                                        score_color,
                                    )>{score_text}</div>
                                </div>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}