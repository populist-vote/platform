use async_openai::types::{
    ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role,
};
use async_openai::Client;
use clap::Parser;
use colored::*;
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::time::Instant;
use uuid::Uuid;

async fn generate_ai_summaries(session_id: Uuid) -> Result<(), Box<dyn Error>> {
    let mut sp = Spinner::new(Spinners::Dots5, "Machine is summarizing bills...".into());
    let start = Instant::now();
    db::init_pool().await.unwrap();
    let db_pool = &db::pool().await.connection;

    let bills = sqlx::query!(
        r#"
            SELECT id, full_text_url, legiscan_data
            FROM bill
            WHERE session_id = $1
            AND populist_summary IS NULL
        "#,
        session_id
    )
    .fetch_all(db_pool)
    .await?;

    for bill in bills {
        let full_text_url = bill.full_text_url;

        if full_text_url.is_none() {
            continue;
        }

        // Download and extract PDF content
        let response = reqwest::get(full_text_url.clone().unwrap()).await?;
        let pdf_url = if response
            .headers()
            .get("content-type")
            .map_or(false, |v| v != "application/pdf")
        {
            let html = response.text().await?;
            let document = scraper::Html::parse_document(&html);
            let selector =
                scraper::Selector::parse("td[data-label='Documents'] a[href$='.pdf']").unwrap();
            if let Some(element) = document.select(&selector).next() {
                element.value().attr("href").unwrap().to_string()
            } else {
                eprintln!(
                    "Failed to find PDF link in the web page for bill with id: {}",
                    bill.id
                );
                continue;
            }
        } else {
            full_text_url.unwrap()
        };

        let pdf = reqwest::get(pdf_url.clone()).await?.bytes().await?;
        let content = match pdf_extract::extract_text_from_mem(&pdf) {
            Ok(text) => text,
            Err(e) => {
                let id = bill.id;
                eprintln!("Failed to extract text from PDF for bill with id: {id} :\n {e}");
                continue;
            }
        };

        let messages = vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are an expert at analyzing legislative text. Provide clear, concise answers based on the bill content provided.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(format!(
                    "Summarize the following bill text clearly and concisely:\n\n{}",
                    content
                ))
                .build()?,
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo-0125")
            .messages(messages)
            .build()?;

        let response = Client::new().chat().create(request).await?;
        let summary = response.choices.first().unwrap().message.content.clone();

        sqlx::query!(
            r#"
                UPDATE bill
                SET pdf_url = $1, populist_summary = $2
                WHERE id = $3
            "#,
            pdf_url,
            summary,
            bill.id
        )
        .execute(db_pool)
        .await?;
    }

    sp.stop();
    let duration = start.elapsed();
    println!("{}", "✅ Bill loaded successfully!".green());
    println!("🕑 {:?}", duration);

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    session_id: Uuid,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = generate_ai_summaries(args.session_id).await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
