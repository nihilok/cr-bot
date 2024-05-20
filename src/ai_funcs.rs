use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use futures::StreamExt;
use std::io::{stdout, Write};

const COMPLETION_TOKENS: u16 = 1024;

const SYSTEM_MESSAGE: &'static str = include_str!("system-message.txt");
const PR_SYSTEM_MESSAGE: &'static str = include_str!("pr-system-message.txt");

pub async fn code_review(output: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = async_openai::Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model("gpt-4-1106-preview")
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

pub async fn implementation_details(output: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = async_openai::Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model("gpt-4-1106-preview")
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
