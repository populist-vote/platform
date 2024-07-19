use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, CreateChatCompletionRequestArgs,
};
use serde_json::Value as JSON;

pub async fn translate_text(
    text: &str,
    languages: Vec<&str>,
) -> Result<JSON, Box<dyn std::error::Error + Send>> {
    let client = async_openai::Client::new();

    let prompt = format!(
        r#"
                    Translate the following text into the following languages: {languages}
                    Text: {text}

                    Response should be a JSON object with the language code as the key and the translation as the value.
                "#,
        languages = languages.join(", "),
        text = text
    );

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([ChatCompletionRequestSystemMessageArgs::default()
            .content(prompt)
            .build()
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    let suggested_translations: serde_json::Value = serde_json::from_str(
        response.choices[0]
            .message
            .content
            .as_deref()
            .unwrap_or_default(),
    )
    .unwrap();

    Ok(suggested_translations)
}
