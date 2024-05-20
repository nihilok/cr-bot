use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionResponseStream,
    CreateChatCompletionRequestArgs,
};
use futures::StreamExt;
use std::env;
use std::io::{stdout, Write};

const COMPLETION_TOKENS: u16 = 1024;

const SYSTEM_MESSAGE: &'static str = include_str!("system-message.txt");
const PR_SYSTEM_MESSAGE: &'static str = include_str!("pr-system-message.txt");

const MODEL: &'static str = "gpt-4o";

const OPENAI_API_KEY_VAR_NAME: &'static str = "CR_BOT_OPENAI_API_KEY";

fn get_client() -> async_openai::Client<OpenAIConfig> {
    let token = env::var(OPENAI_API_KEY_VAR_NAME);
    match token {
        Ok(token) => async_openai::Client::with_config(OpenAIConfig::new().with_api_key(token)),
        Err(_) => async_openai::Client::new(),
    }
}

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
pub async fn code_review(output: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model(MODEL)
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
                .content(output.as_str())
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    print_stream(&mut stream).await
}

pub async fn implementation_details(output: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model(MODEL)
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
                .content(output.as_str())
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    print_stream(&mut stream).await
}
