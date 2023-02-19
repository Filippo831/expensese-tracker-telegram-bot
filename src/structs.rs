use google_sheets4::Sheets;
use google_sheets4::{hyper, hyper_rustls};

#[derive(Clone, Debug)]
pub struct PagamentoStruct {
    pub title: String,
    pub amount: f32,
    pub date: u8,
    pub category: String,
    pub wallet: String,
    pub notes: String,
}

#[derive(Clone, Debug)]
pub struct GuadagnoStruct {
    pub title: String,
    pub amount: f32,
    pub date: u8,
}

impl PagamentoStruct {
    pub fn new() -> Self {
        Self {
            title: "".to_string(),
            amount: 0.0,
            date: 0,
            category: "".to_string(),
            wallet: "".to_string(),
            notes: "".to_string(),
        }
    }
}
impl Default for PagamentoStruct {
    fn default() -> Self {
        Self::new()
    }
}

impl GuadagnoStruct {
    pub fn new() -> Self {
        Self {
            title: "".to_string(),
            amount: 0.0,
            date: 0,
        }
    }
}
impl Default for GuadagnoStruct {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct SheetData {
    pub sheet: Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    pub sheet_id: String,
}
impl SheetData {
    pub fn new(
        sheet: Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
        sheet_id: String,
    ) -> Self {
        Self { sheet, sheet_id }
    }
}
