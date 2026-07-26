use async_openai::types::{
    ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role,
};
use async_openai::Client;
use clap::Parser;
use colored::*;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;
use uuid::Uuid;

async fn generate_ai_summaries(session_id: Uuid) -> Result<(), Box<dyn Error>> {
    let mut sp = Spinner::new(Spinners::Dots5, "Machine is summarizing bills...".into());
    let start = Instant::now();
    println!(
        "\n🚀 Starting bill summary generation for session: {}",
        session_id
    );

    db::init_pool().await.unwrap();
    let db_pool = &db::pool().await.connection;
    println!("\n📊 Database connection established");

    let bills = sqlx::query!(
        r#"
            SELECT id, full_text_url, pdf_url, legiscan_data
            FROM bill
            WHERE session_id = $1
            AND populist_summary IS NULL
        "#,
        session_id
    )
    .fetch_all(db_pool)
    .await?;

    println!("\n📝 Found {} bills to process", bills.len());

    for (index, bill) in bills.iter().enumerate() {
        println!(
            "\n⏳ Processing bill {}/{}: ID {}",
            index + 1,
            bills.len(),
            bill.id
        );

        let pdf_url = if let Some(url) = &bill.pdf_url {
            url.clone()
        } else {
            let full_text_url = bill.full_text_url.clone();

            if full_text_url.is_none() {
                println!("\n⚠️  Skipping bill {} - no full text URL", bill.id);
                continue;
            }

            println!("\n📥 Downloading PDF for bill {}", bill.id);
            // Download and extract PDF content
            let response = reqwest::get(full_text_url.clone().unwrap()).await?;
            if response
                .headers()
                .get("content-type")
                .is_some_and(|v| v != "application/pdf")
            {
                println!("\n🔍 PDF not directly accessible, searching in webpage...");
                let html = response.text().await?;
                let document = scraper::Html::parse_document(&html);
                let selector =
                    scraper::Selector::parse("td[data-label='Documents'] a[href$='.pdf']").unwrap();
                if let Some(element) = document.select(&selector).next() {
                    element.value().attr("href").unwrap().to_string()
                } else {
                    eprintln!(
                        "\n❌ Failed to find PDF link in the web page for bill with id: {}",
                        bill.id
                    );
                    continue;
                }
            } else {
                full_text_url.unwrap()
            }
        };

        println!("\n📄 Extracting text from PDF for bill {}", bill.id);
        let pdf = reqwest::get(pdf_url.clone()).await?.bytes().await?;
        let content = match pdf_extract::extract_text_from_mem(&pdf) {
            Ok(text) => {
                println!("\n✅ Successfully extracted {} characters", text.len());
                text
            }
            Err(e) => {
                let id = bill.id;
                eprintln!("\n❌ Failed to extract text from PDF for bill with id: {id} :\n {e}");
                continue;
            }
        };

        let max_tokens = 16385;
        let truncated_content = if content.len() > max_tokens * 4 {
            println!(
                "\n✂️  Truncating content from {} to {} characters",
                content.len(),
                max_tokens * 4
            );
            content[..max_tokens * 4].to_string()
        } else {
            content
        };

        println!("\n🤖 Generating AI summary for bill {}", bill.id);
        let messages = vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are an expert at analyzing legislative text. Provide clear, concise answers based on the bill content provided.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(format!(
                    "Summarize the following bill text clearly and concisely:\n\n{}",
                    truncated_content
                ))
                .build()?,
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo-0125")
            .messages(messages)
            .build()?;

        let response = Client::new().chat().create(request).await?;
        let summary = response.choices.first().unwrap().message.content.clone();

        println!("\n💾 Saving summary for bill {}", bill.id);
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

        println!(
            "\n✅ Successfully processed bill {}/{}",
            index + 1,
            bills.len()
        );
    }

    sp.stop();
    let duration = start.elapsed();
    println!("{}", "\n✅ All bills processed successfully!".green());
    println!("\n🕑 Total duration: {:?}", duration);

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
