use crate::ApplicationContext;


#[crate::command(slash_command)]
pub async fn ping(ctx: ApplicationContext<'_>) -> Result<(), crate::CustomError> {
    todo!()
}