use twilight_model::application::interaction::ApplicationCommand;

pub struct AutoCompleteContext {
    // pub(crate) http: &'a mut Http,
    pub(crate) command: ApplicationCommand,
}
