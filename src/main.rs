use std::num::{ParseFloatError, ParseIntError};

use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

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
                .branch(case![Command::Pagamento].endpoint(pagamento))
                .branch(case![Command::Guadagno].endpoint(guadagno))
                .branch(case![Command::Link].endpoint(link)),
        )
        .branch(case![Command::Cancel].endpoint(cancel))
        .endpoint(no_command);

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![MainState::PTitle { p_data }].endpoint(pagamento_title))
        .branch(case![MainState::PAmount { p_data }].endpoint(pagamento_amount))
        .branch(case![MainState::PDate { p_data, sheet_data }].endpoint(pagamento_date))
        .branch(case![MainState::PNotes { p_data, sheet_data }].endpoint(pagamento_notes))
        .branch(case![MainState::GTitle { g_data }].endpoint(guadagno_title))
        .branch(case![MainState::GAmount { g_data }].endpoint(guadagno_amount))
        .branch(case![MainState::GDate { g_data, sheet_data }].endpoint(guadagno_date))
        .branch(case![MainState::GetLink].endpoint(get_link));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![MainState::PCategory { p_data, sheet_data }].endpoint(pagamento_category))
        .branch(case![MainState::PWallet { p_data, sheet_data }].endpoint(pagamento_wallet));

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

async fn pagamento(bot: Bot, dialogue: MainDialogue, msg: Message) -> HandlerResult {
    let sheet_id: String = sheet_api::get_sheet_id(msg.chat.id);
    dbg!(&sheet_id);
    if sheet_id == "null" || sheet_id == "" {
        bot.send_message(msg.chat.id, "manda link del foglio")
            .await?;
        dialogue.update(MainState::GetLink).await?;
    } else {
        let p_data = Box::new(structs::PagamentoStruct::new());
        bot.send_message(msg.chat.id, "titolo pagamento").await?;
        dialogue.update(MainState::PTitle { p_data }).await?;
    }
    Ok(())
}

async fn pagamento_title(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    mut p_data: Box<structs::PagamentoStruct>,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "quanto").await?;
            p_data.title = text.to_string();
            dialogue.update(MainState::PAmount { p_data }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "manda un testo").await?;
        }
    }
    Ok(())
}

async fn pagamento_amount(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    mut p_data: Box<structs::PagamentoStruct>,
) -> HandlerResult {
    let hub = sheet_api::api_init().await;
    let sheet_id: String = sheet_api::get_sheet_id(msg.chat.id);
    if sheet_id.is_empty() {
        bot.send_message(msg.chat.id, "manda link del foglio")
            .await?;
        dialogue.exit().await?;
    }
    let sheet_data = Box::new(structs::SheetData::new(hub, sheet_id));
    let value: Result<f32, ParseFloatError> = msg.text().unwrap().parse::<f32>();

    match value {
        Ok(text) => {
            bot.send_message(msg.chat.id, "data").await?;
            p_data.amount = text;
            dialogue
                .update(MainState::PDate { p_data, sheet_data })
                .await?;
        }
        Err(err) => {
            dbg!(err);
            bot.send_message(msg.chat.id, "la data deve essere un numero")
                .await?;
        }
    }
    Ok(())
}

async fn pagamento_date(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    (mut p_data, sheet_data): (Box<structs::PagamentoStruct>, Box<structs::SheetData>),
) -> HandlerResult {
    let value: Result<u8, ParseIntError> = msg.text().unwrap().parse::<u8>();
    match value {
        Ok(text) => {
            let categories_data =
                sheet_api::get_categories(&sheet_data.sheet, &sheet_data.sheet_id).await;

            p_data.date = text;

            let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
            for categories in categories_data.chunks(1) {
                let row = categories
                    .iter()
                    .map(|category| {
                        InlineKeyboardButton::callback(category.to_owned(), category.to_owned())
                    })
                    .collect();
                keyboard.push(row);
            }

            let categories_keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::new(keyboard);

            bot.send_message(msg.chat.id, "Select a product:")
                .reply_markup(categories_keyboard)
                .await?;

            dialogue
                .update(MainState::PCategory { p_data, sheet_data })
                .await?;
        }
        Err(err) => {
            dbg!(err);
            bot.send_message(msg.chat.id, "manda un testo").await?;
        }
    }
    Ok(())
}
async fn pagamento_category(
    bot: Bot,
    dialogue: MainDialogue,
    (mut p_data, sheet_data): (Box<structs::PagamentoStruct>, Box<structs::SheetData>),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(category) = &q.data {
        p_data.category = category.to_string();
        let wallet_sheet = sheet_api::get_wallets(&sheet_data.sheet, &sheet_data.sheet_id).await;
        let wallets = wallet_sheet
            .iter()
            .map(|wallet| InlineKeyboardButton::callback(wallet, wallet));

        bot.send_message(dialogue.chat_id(), "wallet")
            .reply_markup(InlineKeyboardMarkup::new([wallets]))
            .await?;
        dialogue
            .update(MainState::PWallet { p_data, sheet_data })
            .await?;
    }

    Ok(())
}

async fn pagamento_wallet(
    bot: Bot,
    dialogue: MainDialogue,
    (mut p_data, sheet_data): (Box<structs::PagamentoStruct>, Box<structs::SheetData>),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(wallet) = &q.data {
        p_data.wallet = wallet.to_string();
        bot.send_message(dialogue.chat_id(), "note aggiuntive")
            .await?;

        dialogue
            .update(MainState::PNotes { p_data, sheet_data })
            .await?;
    }

    Ok(())
}

async fn pagamento_notes(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    (mut p_data, sheet_data): (Box<structs::PagamentoStruct>, Box<structs::SheetData>),
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "finito").await?;
            p_data.notes = text.to_string();
            dbg!(&p_data);
            sheet_api::write_pagamento_data(&sheet_data.sheet, &sheet_data.sheet_id, p_data).await;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "manda un testo").await?;
        }
    }
    Ok(())
}

async fn guadagno(bot: Bot, dialogue: MainDialogue, msg: Message) -> HandlerResult {
    let sheet_id: String = sheet_api::get_sheet_id(msg.chat.id);
    if sheet_id == "null" || sheet_id == "" {
        bot.send_message(msg.chat.id, "manda link del foglio")
            .await?;
        dialogue.update(MainState::GetLink).await?;
    } else {
        let g_data = Box::new(structs::GuadagnoStruct::new());
        bot.send_message(msg.chat.id, "titolo guadagno").await?;
        dialogue.update(MainState::GTitle { g_data }).await?;
    }
    Ok(())
}

async fn guadagno_title(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    mut g_data: Box<structs::GuadagnoStruct>,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "quanto").await?;
            g_data.title = text.to_string();
            dialogue.update(MainState::GAmount { g_data }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "manda un testo").await?;
        }
    }
    Ok(())
}

async fn guadagno_amount(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    mut g_data: Box<structs::GuadagnoStruct>,
) -> HandlerResult {
    let value: Result<f32, ParseFloatError> = msg.text().unwrap().parse::<f32>();
    let hub = sheet_api::api_init().await;
    let sheet_id: String = sheet_api::get_sheet_id(msg.chat.id);
    if sheet_id.is_empty() {
        bot.send_message(msg.chat.id, "manda link del foglio")
            .await?;
        dialogue.exit().await?;
    }
    let sheet_data = Box::new(structs::SheetData::new(hub, sheet_id));

    match value {
        Ok(text) => {
            bot.send_message(msg.chat.id, "data").await?;
            g_data.amount = text;
            dialogue
                .update(MainState::GDate { g_data, sheet_data })
                .await?;
        }
        Err(err) => {
            dbg!(err);
            bot.send_message(msg.chat.id, "manda un testo").await?;
        }
    }
    Ok(())
}
async fn guadagno_date(
    bot: Bot,
    dialogue: MainDialogue,
    msg: Message,
    (mut g_data, sheet_data): (Box<structs::GuadagnoStruct>, Box<structs::SheetData>),
) -> HandlerResult {
    let value: Result<u8, ParseIntError> = msg.text().unwrap().parse::<u8>();
    match value {
        Ok(text) => {
            bot.send_message(msg.chat.id, "finito").await?;
            g_data.date = text;
            dbg!(&g_data);
            sheet_api::write_guadagno_data(&sheet_data.sheet, &sheet_data.sheet_id, g_data).await;
            dialogue.exit().await?;
        }
        Err(err) => {
            dbg!(err);
            bot.send_message(msg.chat.id, "manda un testo").await?;
        }
    }
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
