use crate::translate::gemini::translate as gemini;
use crate::translate::open_ai::translate as open_ai;
pub struct Context {
    pub model: String,
    pub api_key: String,
    pub language: String,
    pub lines: usize,
    pub requests: usize,
}

pub enum Translator {
    OpenAi(Context),
    Gemini(Context),
}

impl Translator {
    pub async fn translate(&self, lines: Vec<String>) -> Vec<String> {
        match self {
            Self::OpenAi(context) => open_ai(context, lines).await,
            Self::Gemini(context) => gemini(context, lines).await,
        }
    }
}
