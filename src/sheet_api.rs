use chrono::Datelike;
use chrono::Local;
use google_sheets4::oauth2;
use google_sheets4::Sheets;
use google_sheets4::{api::ValueRange, hyper, hyper_rustls, Error};
use std::fs;
use teloxide::types::ChatId;

use crate::structs;

static MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub async fn api_init() -> Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    let secret = oauth2::read_service_account_key("service-account.json")
        .await
        .expect("credential not read");

    let auth = oauth2::ServiceAccountAuthenticator::builder(secret)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();

    let hub = Sheets::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );

    return hub;
}

pub async fn get_pagamenti_empty_cell(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: &str,
) -> u32 {
    let month_number = Local::now().month0() as usize;

    let range = format!("{}!B4:B1000", MONTHS[month_number]);

    let response = hub
        .spreadsheets()
        .values_get(sheet_id, &range)
        .value_render_option("UNFORMATTED_VALUE")
        .major_dimension("COLUMNS")
        .doit()
        .await;

    let values = match response {
        Ok((_response, values)) => values,
        Err(error) => {
            println!("Error getting values: {}", error);
            return None.unwrap();
        }
    };

    if values.values.is_none() {
        return 4;
    }

    return 4 + values.values.unwrap().get(0).unwrap().len() as u32;
}

pub async fn get_categories(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: &str,
) -> Vec<String> {
    let range = "Categories!B4:B20";

    let response = hub
        .spreadsheets()
        .values_get(sheet_id, range)
        .value_render_option("UNFORMATTED_VALUE")
        .major_dimension("COLUMNS")
        .doit()
        .await;

    let values = match response {
        Ok((_response, values)) => values,
        Err(error) => {
            println!("Error getting values: {}", error);
            return None.unwrap();
        }
    };
    return values.values.unwrap().get(0).unwrap().to_vec();
}

pub async fn get_wallets(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: &str,
) -> Vec<String> {
    let range = "Categories!G4:G20";

    let response = hub
        .spreadsheets()
        .values_get(sheet_id, range)
        .value_render_option("UNFORMATTED_VALUE")
        .major_dimension("COLUMNS")
        .doit()
        .await;

    let values = match response {
        Ok((_response, values)) => values,
        Err(error) => {
            println!("Error getting values: {}", error);
            return None.unwrap();
        }
    };
    return values.values.unwrap().get(0).unwrap().to_vec();
}

pub async fn write_pagamento_data(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: &str,
    data: Box<structs::PagamentoStruct>,
) {
    let row = get_pagamenti_empty_cell(hub, sheet_id).await.to_string();

    let month_number = Local::now().month0() as usize;

    let range = format!("{}!B{}:G{}", MONTHS[month_number], row, row);

    let values_vector = vec![vec![
        data.title,
        data.amount.to_string(),
        data.date.to_string(),
        data.category,
        data.wallet,
        data.notes,
    ]];

    let values = ValueRange {
        values: Some(values_vector),
        ..Default::default()
    };

    let response = hub
        .spreadsheets()
        .values_update(values, sheet_id, &range)
        .value_input_option("USER_ENTERED")
        .doit()
        .await;

    match response {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success {:?}", res),
    }
}
pub async fn get_guadagni_empty_cell(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: &str,
) -> u32 {
    let month_number = Local::now().month0() as usize;

    let range = format!("{}!I4:I1000", MONTHS[month_number]);

    let response = hub
        .spreadsheets()
        .values_get(sheet_id, &range)
        .value_render_option("UNFORMATTED_VALUE")
        .major_dimension("COLUMNS")
        .doit()
        .await;

    let values = match response {
        Ok((_response, values)) => values,
        Err(error) => {
            println!("Error getting values: {}", error);
            return None.unwrap();
        }
    };

    if values.values.is_none() {
        return 4;
    }

    return 4 + values.values.unwrap().get(0).unwrap().len() as u32;
}

pub async fn write_guadagno_data(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: &str,
    data: Box<structs::GuadagnoStruct>,
) {
    let row = get_guadagni_empty_cell(hub, sheet_id).await.to_string();

    let month_number = Local::now().month0() as usize;

    let range = format!("{}!I{}:K{}", MONTHS[month_number], row, row);

    let values_vector = vec![vec![
        data.title,
        data.amount.to_string(),
        data.date.to_string(),
    ]];

    let values = ValueRange {
        values: Some(values_vector),
        ..Default::default()
    };

    let response = hub
        .spreadsheets()
        .values_update(values, sheet_id, &range)
        .value_input_option("USER_ENTERED")
        .doit()
        .await;

    match response {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success {:?}", res),
    }
}
pub fn write_sheet_id(sheet_id: String, chat_id: ChatId) {
    let file_data = fs::read_to_string("user_data.json").unwrap();
    let mut json_data: serde_json::Value = serde_json::from_str(&file_data).unwrap();

    let key: &str = sheet_id.split('/').nth(5).unwrap();

    json_data[chat_id.to_string()] = serde_json::json!(key);

    fs::write("user_data.json", json_data.to_string()).expect("no buono");
}

pub async fn check_sheet_id(
    sheet_id: String,
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
) -> bool {
    let key: &str = sheet_id.split('/').nth(5).unwrap();
    let result = hub.spreadsheets().get(key).doit().await;
    return result.is_ok();
}

pub fn get_sheet_id(chat_id: ChatId) -> String {
    let file_data = fs::read_to_string("user_data.json").unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&file_data).unwrap();
    if let Some(sheet_id) = json_data.get(chat_id.to_string()) {
        if let Some(sheet_id_string) = sheet_id.as_str() {
            return sheet_id_string.to_string();
        }
    }
    return "".to_string();
}
