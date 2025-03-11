use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use clap::Parser;
use colored::*;
use db::BillStatus;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::fmt;
use std::io::{self, Write};
use std::time::Instant;
use tokio;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bill slug for lookup
    #[arg(short, long)]
    slug: String,
}

// Additional context added beyond the full text
#[derive(Debug)]
struct BillMeta {
    status: BillStatus,
}

struct BillContext {
    meta: BillMeta,
    body: String,
    client: Client,
}

impl fmt::Display for BillMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status: {:?}", self.status)
    }
}

impl BillContext {
    async fn new(slug: &str) -> Result<Self, Box<dyn Error>> {
        let mut sp = Spinner::new(Spinners::Dots5, "Loading bill content...".into());

        db::init_pool().await.unwrap();
        let db_pool = &db::pool().await.connection;
        let bill = db::Bill::find_by_slug(db_pool, slug).await?;
        let pdf_url = bill.pdf_url;

        if pdf_url.is_none() {
            return Ok(Self {
                meta: BillMeta {
                    status: bill.status,
                },
                body: "".into(),
                client: Client::new(),
            });
        }

        // Download and extract PDF content
        let pdf = reqwest::get(pdf_url.unwrap()).await?.bytes().await?;
        let content = pdf_extract::extract_text_from_mem(&pdf)?;

        sp.stop();
        println!("{}", "✅ Bill loaded successfully!".green());

        Ok(Self {
            meta: BillMeta {
                status: bill.status,
            },
            body: content,
            client: Client::new(),
        })
    }

    async fn ask_question(&self, question: &str) -> Result<String, Box<dyn Error>> {
        let mut sp = Spinner::new(Spinners::Dots5, "Getting response...".into());

        let messages = vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are an expert at analyzing legislative text. Provide clear, concise answers based on the bill content provided.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(format!(
                    "A bill has the following metadata:\n{}\n
                    Based on this legislative bill text:\n\n{}\n\nAnswer this question: {}",
                    self.meta, self.body, question
                ))
                .build()?,
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo-0125")
            .messages(messages)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let answer = response.choices.first().unwrap().message.content.clone();

        sp.stop();
        Ok(answer)
    }
}

async fn run_chat_interface(slug: &str) -> Result<(), Box<dyn Error>> {
    println!("{}", "\n📜 Bill Chat Interface".bold());
    println!("Type your questions about the bill (or 'exit' to quit)\n");

    let context = BillContext::new(slug).await?;

    loop {
        print!("{}", "\n❔ Your question: ".cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let question = input.trim();

        if question.eq_ignore_ascii_case("exit") {
            println!("{}", "\nGoodbye! 👋".green());
            break;
        }

        if question.is_empty() {
            continue;
        }

        let start = Instant::now();
        match context.ask_question(question).await {
            Ok(answer) => {
                println!("\n{}", "Answer:".blue().bold());
                println!("{}", answer.trim());
                println!("\n🕑 Response time: {:?}", start.elapsed());
            }
            Err(e) => {
                eprintln!("\n{} {}", "Error:".red().bold(), e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    run_chat_interface(&args.slug).await
}
