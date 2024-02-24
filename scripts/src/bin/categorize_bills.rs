use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, CreateChatCompletionRequestArgs,
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn categorize_bills() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let start = Instant::now();

    let bill_records = sqlx::query!(
        r#"
        SELECT b.id, b.title, issue_tag_id FROM bill b 
        LEFT JOIN bill_issue_tags ON b.id = bill_issue_tags.bill_id
        WHERE (attributes->>'categorized')::boolean IS NOT true
        AND bill_issue_tags.issue_tag_id IS NULL;
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    println!("\nCategorizing {} bills... \n", bill_records.len());

    let issue_tags = sqlx::query!(r#"SELECT id, slug FROM issue_tag"#)
        .fetch_all(&pool.connection)
        .await?;

    let client = async_openai::Client::new();

    let bar = ProgressBar::new(bill_records.len() as u64);
    bar.set_style(
        ProgressStyle::with_template("🕑 {elapsed_precise} {bar:60.cyan/blue} {pos}/{len}")
            .unwrap(),
    );

    for bill in bill_records {
        let prompt = format!(
            r#"
                I have the following categories of bills: {tags}
                Categorize a bill with the following title into one or more of the categories above: {title}
                Respond only with the slugified tags, as listed above, comma separated.
            "#,
            tags = issue_tags
                .iter()
                .map(|tag| tag.slug.clone())
                .collect::<Vec<_>>()
                .join(", "),
            title = bill.title
        );
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-3.5-turbo")
            .messages([ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt)
                .build()?
                .into()])
            .build()?;

        let response = client.chat().create(request).await?;
        let suggested_slugs: Vec<&str> = response.choices[0]
            .message
            .content
            .as_deref()
            .unwrap_or_default()
            .split(",")
            .map(str::trim)
            .collect();

        for slug in suggested_slugs {
            let issue_tag = issue_tags.iter().find(|tag| tag.slug == slug.trim());
            if let Some(tag) = issue_tag {
                sqlx::query!(
                    r#"
                        INSERT INTO bill_issue_tags (bill_id, issue_tag_id) 
                        VALUES ($1, (SELECT id FROM issue_tag WHERE slug = $2)) 
                        ON CONFLICT DO NOTHING
                    "#,
                    bill.id,
                    tag.slug
                )
                .execute(&pool.connection)
                .await?;
            }
        }

        sqlx::query!(
            r#"
                UPDATE bill
                SET attributes = attributes || jsonb_build_object('categorized', true)
                WHERE id = $1
            "#,
            bill.id
        )
        .execute(&pool.connection)
        .await?;

        bar.inc(1);
    }

    bar.finish_and_clear();
    let duration = start.elapsed();
    eprintln!(
        "
✅ {}",
        "Success".bright_green().bold()
    );
    eprintln!(
        "
🕑 {:?}",
        duration
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = categorize_bills().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
