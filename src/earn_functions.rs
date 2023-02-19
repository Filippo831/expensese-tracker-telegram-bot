use std::num::{ParseFloatError, ParseIntError};

use teloxide::{requests::Requester, types::Message, Bot};

use crate::{sheet_api, structs, HandlerResult, MainDialogue, MainState};

pub async fn guadagno(bot: Bot, dialogue: MainDialogue, msg: Message) -> HandlerResult {
    let sheet_id: String = sheet_api::get_sheet_id(msg.chat.id);
    if sheet_id == "null" || sheet_id.is_empty() {
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

pub async fn guadagno_title(
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

pub async fn guadagno_amount(
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
pub async fn guadagno_date(
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
