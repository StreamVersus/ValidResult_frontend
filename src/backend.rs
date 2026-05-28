use std::fmt::Display;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const BACKEND_ADDRESS: &str = "https://oheaven.ru";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    LLM,
    CV,
    CVML,
    LLMML,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::LLM => write!(f, "LLM"),
            MetricType::CV => write!(f, "CV"),
            MetricType::CVML => write!(f, "CVML"),
            MetricType::LLMML => write!(f, "LLMML"),
        }
    }
}

impl MetricType {
    pub const ALL: [Self; 2] = [Self::LLM, Self::CV];
    pub const ALL_ML: [Self; 2] = [Self::LLMML, Self::CVML];

    pub fn from_str(string: &str) -> MetricType {
        match string {
            "LLM" => MetricType::LLM,
            "CV" => MetricType::CV,
            "CVML" => MetricType::CVML,
            "LLMML" => MetricType::LLMML,
            _ => unimplemented!(),
        }
    }

    pub fn to_ml(self) -> Self {
        match self {
            MetricType::LLM => MetricType::LLMML,
            MetricType::CV => MetricType::CVML,
            _ => self
        }
    }

    pub fn metric_labels(&self) -> &'static [&'static str] {
        match self {
            MetricType::LLM => &[
                "Средняя задержка",
                "Точность ответа",
                "Логичность и структура",
                "Глубина и полнота ответа",
                "Гибкость в интерпретации",
                "Адекватность формата",
                "Чувствительность к контексту",
                "Креативность",
                "Адаптивность к стилю общения",
                "Обработка сложных запросов",
                "Восприятие неоднозначности",
                "Оценка релевантности источников",
                "Эффективность исправления ошибок",
                "Понимание технической специфики",
                "Устойчивость к когнитивной нагрузке",
                "Интерактивность",
                "Умение работать с неоднозначными терминами",
                "Анализ и синтез информации",
                "Четкость и краткость",
                "Поддержка мультимодальных запросов",
                "Корректность грамматики и стиля",
                "Способность к самообучению",
                "Этика и корректность",
                "Поддержка многозадачности",
                "Понимание культурных различий",
                "Прогнозирование и предвосхищение",
                "Работа с ограниченными данными",
                "Адаптация к изменениям в диалоге",
                "Размер и сложность генерируемого текста",
                "Обратная связь",
            ],
            MetricType::CV => &[
                "Точность на цели",
                "Точность на неизвестных",
                "Устойчивость к дождю",
                "Устойчивость к разрешению",
                "Скорость инференса",
            ],
            _ => unimplemented!(),
        }
    }
}

pub async fn pull_from_backend(metric_type: MetricType) -> Result<String, String> {
    let url = format!(
        "{}/metrics?request_type={}",
        BACKEND_ADDRESS,
        metric_type.to_string().to_lowercase()
    );

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    response.text().await.map_err(|e| format!("Failed to read response: {e}"))
}

#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    response: String,
    #[serde(default)]
    session_id: String,
}


#[derive(Serialize)]
struct ChatRequest<'a> {
    user_message: &'a str,
    user_metrics: &'a str,
}

pub async fn ai_agent_turn(
    message: String,
    embed: bool,
    metric_type: MetricType,
    session_id: Option<String>,
    user_metrics: String,
) -> (String, String) {
    let mut url = format!(
        "{}/chat?is_embed={}&request_type={}",
        BACKEND_ADDRESS, embed, metric_type.to_string().to_lowercase()
    );
    if let Some(sid) = session_id {
        if !sid.is_empty() {
            url.push_str(&format!("&session_id={}", sid));
        }
    }

    let data: ChatResponse = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&ChatRequest {
            user_message: &message,
            user_metrics: &user_metrics,
        }).expect("Failed to serialize request")
        .send().await.unwrap()
        .json::<ChatResponse>().await.unwrap();

    (data.response, data.session_id)
}