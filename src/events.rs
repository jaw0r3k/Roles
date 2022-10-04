use twilight_model::{
  application::interaction::InteractionData,
  channel::message::MessageFlags,
  gateway::payload::incoming::InteractionCreate,
  http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{commands, State};

#[tracing::instrument(ret, skip_all)]
pub async fn interaction_dispatcher(
  state: State,
  interaction: Box<InteractionCreate>,
) -> anyhow::Result<()> {
  let response = match interaction.data {
    Some(InteractionData::ApplicationCommand(ref command)) => {
      commands::handle_command(state.clone(), command, &interaction).await
    }
    Some(InteractionData::MessageComponent(ref component)) => {
      commands::handle_menu(state.clone(), interaction.clone(), component).await
    }
    _ => unreachable!(),
  }
  .unwrap_or_else(|err| InteractionResponse {
    kind: InteractionResponseType::ChannelMessageWithSource,
    data: Some(
      InteractionResponseDataBuilder::new()
        .content(err.to_string())
        .flags(MessageFlags::EPHEMERAL)
        .build(),
    ),
  });

  let client = state.client.interaction(state.app_id);
  client
    .create_response(interaction.id, &interaction.token, &response)
    .exec()
    .await?
    .text()
    .await?;

  Ok(())
}
