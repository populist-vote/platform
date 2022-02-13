use db::CreateOrConnectIssueTagInput;
use serde_yaml;
use std::{error, fs, io, path, process};

async fn seed() -> Result<(), Box<dyn error::Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let mut dir = path::PathBuf::new();
    dir.push(std::env::var("CARGO_MANIFEST_DIR")?);
    dir.push("../db/seeds");

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let model_name = &path.file_stem().unwrap();
        let file = fs::File::open(path.to_owned())?;
        let reader = io::BufReader::new(file);
        let yaml: serde_yaml::Mapping =
            serde_yaml::from_reader(reader).expect("YAML was improperly formatted");

        match model_name.to_str().unwrap() {
            // "users" => {
            //     for (name, user) in yaml.into_iter() {
            //         let input: db::CreateUserInput = serde_yaml::from_value(user).unwrap();
            //         db::User::create(&pool.connection, &input).await.unwrap();
            //         println!("Created user record for: {}", name.as_str().unwrap());
            //     }
            // }
            "organizations" => {
                for (name, organization) in yaml.into_iter() {
                    let input: db::CreateOrganizationInput =
                        serde_yaml::from_value(organization.to_owned()).unwrap();
                    let record = db::Organization::create(&pool.connection, &input)
                        .await
                        .unwrap();
                    let issue_tag_input: CreateOrConnectIssueTagInput =
                        serde_yaml::from_value(organization["issue_tags"].to_owned()).unwrap();
                    graphql::mutation::organization::handle_nested_issue_tags(
                        &pool.connection,
                        record.id,
                        issue_tag_input,
                    )
                    .await?;
                    println!(
                        "Created organization record for: {}",
                        name.as_str().unwrap()
                    );
                }
            }
            // "issues" => {
            //     for (name, issue) in yaml.into_iter() {
            //         let input: db::CreateIssueTagInput = serde_yaml::from_value(issue).unwrap();
            //         db::IssueTag::create(&pool.connection, &input)
            //             .await
            //             .unwrap();
            //         println!("Created issue tag record for : {}", name.as_str().unwrap())
            //     }
            // }
            _ => (), // panic!("model is not supported: {:?}", model_name),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed().await {
        println!("Error seeding the database: {}", err);
        process::exit(1);
    }
}
