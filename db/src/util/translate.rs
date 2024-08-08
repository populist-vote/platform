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
                    Response should be a parsable JSON object with the language code as the key and the translation as the value.
                    Be sure to escape any special characters in the text, including quotations.
                    Do not include the original text in the response and only translate everything after "Text:".
                    
                    Text: {text}

                "#,
        languages = languages.join(", "),
        text = text
    );

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u16)
        .model("gpt-3.5-turbo")
        .messages([ChatCompletionRequestSystemMessageArgs::default()
            .content(prompt)
            .build()
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    let suggested_translations = serde_json::from_str(
        response.choices[0]
            .message
            .content
            .as_deref()
            .unwrap_or_default(),
    );

    if suggested_translations.is_err() {
        eprintln!(
            "Failed to parse translation response with: {}",
            response.choices[0]
                .message
                .content
                .as_deref()
                .unwrap_or_default()
        );
    }

    let translations: serde_json::Value = suggested_translations.unwrap();

    Ok(translations)
}
