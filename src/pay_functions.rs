use std::num::{ParseFloatError, ParseIntError};

use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    Bot,
};

use crate::{sheet_api, structs, HandlerResult, MainDialogue, MainState};

pub async fn pagamento(bot: Bot, dialogue: MainDialogue, msg: Message) -> HandlerResult {
    let sheet_id: String = sheet_api::get_sheet_id(msg.chat.id);
    dbg!(&sheet_id);
    if sheet_id == "null" || sheet_id.is_empty() {
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

pub async fn pagamento_title(
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

pub async fn pagamento_amount(
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

pub async fn pagamento_date(
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
pub async fn pagamento_category(
    bot: Bot,
    dialogue: MainDialogue,
    (mut p_data, sheet_data): (Box<structs::PagamentoStruct>, Box<structs::SheetData>),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(category) = &q.data {
        bot.answer_callback_query(q.id).await?;
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

pub async fn pagamento_wallet(
    bot: Bot,
    dialogue: MainDialogue,
    (mut p_data, sheet_data): (Box<structs::PagamentoStruct>, Box<structs::SheetData>),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(wallet) = &q.data {
        bot.answer_callback_query(q.id).await?;
        p_data.wallet = wallet.to_string();
        bot.send_message(dialogue.chat_id(), "note aggiuntive")
            .await?;

        dialogue
            .update(MainState::PNotes { p_data, sheet_data })
            .await?;
    }

    Ok(())
}

pub async fn pagamento_notes(
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
