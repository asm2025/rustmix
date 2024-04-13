use fake::{
    faker::automotive::raw as f_automotive,
    locales, Fake
};

pub fn license_number() -> String {
	f_automotive::LicencePlate(locales::FR_FR).fake()
}
