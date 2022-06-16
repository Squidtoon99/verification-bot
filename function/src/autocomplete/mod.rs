use crate::CustomError;
use twilight_model::{
    application::{command::CommandOptionChoice, interaction::ApplicationCommandAutocomplete},
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};
mod context;

pub async fn handle(
    _data: ApplicationCommandAutocomplete,
) -> Result<InteractionResponse, CustomError> {
    Ok(InteractionResponse {
        kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
        data: Some(InteractionResponseData {
            choices: Some(vec![
                CommandOptionChoice::String {
                    name: "A".to_string(),
                    name_localizations: None,
                    value: "A".to_string(),
                },
                CommandOptionChoice::String {
                    name: "B".to_string(),
                    name_localizations: None,
                    value: "B".to_string(),
                },
                CommandOptionChoice::String {
                    name: "C".to_string(),
                    name_localizations: None,
                    value: "C".to_string(),
                },
                CommandOptionChoice::String {
                    name: "D".to_string(),
                    name_localizations: None,
                    value: "D".to_string(),
                },
            ]),
            ..Default::default()
        }),
    })
}
