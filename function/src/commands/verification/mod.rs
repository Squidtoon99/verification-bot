use zephyrus::{prelude::*, twilight_exports::{InteractionResponseType, InteractionResponseData, InteractionResponse}};
#[command]
#[description = "Says hello"]
async fn role(ctx: &SlashContext<crate::Context>,
    #[description = "abc"] abc: String,
) -> CommandResult {
    dbg!(abc);
    Ok(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(String::from("Hello world")),
                    ..Default::default()
                }),
            })
}