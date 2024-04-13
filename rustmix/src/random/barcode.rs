use fake::{
    faker::barcode::raw as f_barcode,
    locales, Fake
};

pub fn isbn() -> String {
	f_barcode::Isbn(locales::EN).fake()
}

pub fn isbn10() -> String {
	f_barcode::Isbn10(locales::EN).fake()
}

pub fn isbn13() -> String {
	f_barcode::Isbn13(locales::EN).fake()
}
