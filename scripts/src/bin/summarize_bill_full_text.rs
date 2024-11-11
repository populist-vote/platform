use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use colored::*;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::io::{self, Write};
use std::time::Instant;
use tokio;

struct BillContext {
    content: String,
    client: Client,
}

impl BillContext {
    async fn new(pdf_url: &str) -> Result<Self, Box<dyn Error>> {
        let mut sp = Spinner::new(Spinners::Dots5, "Loading bill content...".into());

        // Download and extract PDF content
        let pdf = reqwest::get(pdf_url).await?.bytes().await?;
        let content = pdf_extract::extract_text_from_mem(&pdf)?;

        sp.stop();
        println!("{}", "✅ Bill loaded successfully!".green());

        Ok(Self {
            content,
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
                    "Based on this legislative bill text:\n\n{}\n\nAnswer this question: {}",
                    self.content, question
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

async fn run_chat_interface() -> Result<(), Box<dyn Error>> {
    println!("{}", "\n📜 Bill Chat Interface".bold());
    println!("Type your questions about the bill (or 'exit' to quit)\n");

    let full_text_pdf_url =
        "https://legiscan.com/CO/text/HB1384/id/3001444/Colorado-2024-HB1384-Enrolled.pdf";

    let context = BillContext::new(full_text_pdf_url).await?;

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
    run_chat_interface().await
}
