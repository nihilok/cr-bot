use async_openai::{
    config,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionResponseStream,
        CreateChatCompletionRequestArgs,
    },
};
use futures::StreamExt;
use std::env;
use std::io::{stdout, Write};

const COMPLETION_TOKENS: u16 = 1024;

const SYSTEM_MESSAGE: &'static str = include_str!("system-message.txt");
const PR_SYSTEM_MESSAGE: &'static str = include_str!("pr-system-message.txt");

const MODEL_VAR_NAME: &'static str = "CR_BOT_MODEL_NAME";
const DEFAULT_MODEL: &'static str = "gpt-4o-mini";

const OPENAI_API_KEY_VAR_NAME: &'static str = "CR_BOT_OPENAI_API_KEY";

/// Helper function to create an OpenAI client using the appropriate API key
fn get_client() -> async_openai::Client<config::OpenAIConfig> {
    let token = env::var(OPENAI_API_KEY_VAR_NAME);
    match token {
        Ok(token) => {
            async_openai::Client::with_config(config::OpenAIConfig::new().with_api_key(token))
        }
        Err(_) => {
            println!("No '{}' environment variable supplied; falling back to default 'OPENAI_API_KEY' environment variable.", OPENAI_API_KEY_VAR_NAME);
            async_openai::Client::new()
        }
    }
}

/// Print stream to stdout as it is returned (does not wait for full response before starting printing)
async fn print_stream(
    stream: &mut ChatCompletionResponseStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = stdout().lock();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                    if let Some(ref content) = chat_choice.delta.content {
                        if let Err(e) = write!(lock, "{}", content) {
                            return Err(e.into());
                        }
                    }
                }
            }
            Err(err) => return Err(err.into()),
        }
        stdout().flush()?;
    }
    Ok(())
}

fn get_model_name() -> String {
    env::var(MODEL_VAR_NAME).unwrap_or(DEFAULT_MODEL.to_owned())
}


/// Review PR changes (or local changes on current branch) supplied as `input`
pub async fn code_review(input: String) -> Result<(), Box<dyn std::error::Error>> {
    let model = get_model_name();
    let client = get_client();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model(model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(SYSTEM_MESSAGE)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("Can you review some changes from a PR?")
                .build()?
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content("Sure thing! What are the changes?")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(input.as_str())
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    print_stream(&mut stream).await
}

/// Describe PR changes (or local changes on current branch) supplied as `input`
pub async fn implementation_details(input: String) -> Result<(), Box<dyn std::error::Error>> {
    let model = get_model_name();
    let client = get_client();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model(model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(PR_SYSTEM_MESSAGE)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(
                    "Can please provide the implementation details for the following PR changes?",
                )
                .build()?
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content("Sure thing! What are the changes?")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(input.as_str())
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    print_stream(&mut stream).await
}
