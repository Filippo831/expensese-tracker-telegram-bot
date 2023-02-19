use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

mod earn_functions;
mod pay_functions;
mod sheet_api;
mod structs;

type MainDialogue = Dialogue<MainState, InMemStorage<MainState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum MainState {
    #[default]
    Start,
    PTitle {
        p_data: Box<structs::PagamentoStruct>,
    },
    PAmount {
        p_data: Box<structs::PagamentoStruct>,
    },
    PDate {
        p_data: Box<structs::PagamentoStruct>,
        sheet_data: Box<structs::SheetData>,
    },
    PCategory {
        p_data: Box<structs::PagamentoStruct>,
        sheet_data: Box<structs::SheetData>,
    },
    PWallet {
        p_data: Box<structs::PagamentoStruct>,
        sheet_data: Box<structs::SheetData>,
    },
    PNotes {
        p_data: Box<structs::PagamentoStruct>,
        sheet_data: Box<structs::SheetData>,
    },
    GTitle {
        g_data: Box<structs::GuadagnoStruct>,
    },
    GAmount {
        g_data: Box<structs::GuadagnoStruct>,
    },
    GDate {
        g_data: Box<structs::GuadagnoStruct>,
        sheet_data: Box<structs::SheetData>,
    },
    GetLink,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Command supported")]
pub enum Command {
    #[command(description = "mostra help")]
    Help,
    #[command(description = "pagamento")]
    Pagamento,
    #[command(description = "guadagno")]
    Guadagno,
    #[command(description = "spreadsheet link")]
    Link,
    #[command(description = "cancella")]
    Cancel,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("starting bot");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<MainState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;
    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![MainState::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Pagamento].endpoint(pay_functions::pagamento))
                .branch(case![Command::Guadagno].endpoint(earn_functions::guadagno))
                .branch(case![Command::Link].endpoint(link)),
        )
        .branch(case![Command::Cancel].endpoint(cancel))
        .endpoint(no_command);

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![MainState::PTitle { p_data }].endpoint(pay_functions::pagamento_title))
        .branch(case![MainState::PAmount { p_data }].endpoint(pay_functions::pagamento_amount))
        .branch(
            case![MainState::PDate { p_data, sheet_data }].endpoint(pay_functions::pagamento_date),
        )
        .branch(
            case![MainState::PNotes { p_data, sheet_data }]
                .endpoint(pay_functions::pagamento_notes),
        )
        .branch(case![MainState::GTitle { g_data }].endpoint(earn_functions::guadagno_title))
        .branch(case![MainState::GAmount { g_data }].endpoint(earn_functions::guadagno_amount))
        .branch(
            case![MainState::GDate { g_data, sheet_data }].endpoint(earn_functions::guadagno_date),
        )
        .branch(case![MainState::GetLink].endpoint(get_link));

    let callback_query_handler = Update::filter_callback_query()
        .branch(
            case![MainState::PCategory { p_data, sheet_data }]
                .endpoint(pay_functions::pagamento_category),
        )
        .branch(
            case![MainState::PWallet { p_data, sheet_data }]
                .endpoint(pay_functions::pagamento_wallet),
        );

    dialogue::enter::<Update, InMemStorage<MainState>, MainState, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn no_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "comando non valido").await?;
    Ok(())
}

async fn cancel(bot: Bot, msg: Message, dialogue: MainDialogue) -> HandlerResult {
    dialogue.exit().await?;
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn link(bot: Bot, dialogue: MainDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "manda link del file").await?;
    dialogue.update(MainState::GetLink).await?;
    Ok(())
}

async fn get_link(bot: Bot, dialogue: MainDialogue, msg: Message) -> HandlerResult {
    dialogue.exit().await?;
    let hub = sheet_api::api_init().await;
    match msg.text() {
        Some(text) => {
            if sheet_api::check_sheet_id(text.to_string(), &hub).await {
                sheet_api::write_sheet_id(text.to_string(), msg.chat.id);
                bot.send_message(msg.chat.id, "ok").await?;
                dialogue.exit().await?;
            } else {
                bot.send_message(msg.chat.id, "manda un link").await?;
                dialogue.update(MainState::GetLink).await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "manda un link").await?;
            dialogue.update(MainState::GetLink).await?;
        }
    }
    Ok(())
}
