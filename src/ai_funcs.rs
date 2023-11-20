use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use futures::StreamExt;
use std::io::{stdout, Write};

const COMPLETION_TOKENS: u16 = 1024;

pub async fn code_review(output: String) -> Result<(), Box<dyn std::error::Error>> {
    let system_message: &str = "\
You are a code reviewer. \
You provide your response in markdown, \
using a heading (`## path/filename.ext`) for each file reviewed; \
normal text for your comment; and, potentially, code blocks for \
code snippets relating to suggested changes (```language...\n```). \
Don't bother commenting on everything, just focus on things you think \
would benefit from being reworked. Very occasionally, you might add \
positive comments about things that are particularly well executed, \
but this is entirely optional.";

    let client = async_openai::Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model("gpt-4-1106-preview")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_message)
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
