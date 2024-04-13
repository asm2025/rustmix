use fake::{
	faker::address::raw as f_address,
	locales, Fake
};

pub fn building() -> String {
	f_address::BuildingNumber(locales::EN).fake()
}

pub fn street() -> String {
	f_address::StreetName(locales::EN).fake()
}

pub fn street_suffix() -> String {
	f_address::StreetSuffix(locales::EN).fake()
}

pub fn secondary_address() -> String {
	f_address::SecondaryAddress(locales::EN).fake()
}

pub fn secondary_address_type() -> String {
	f_address::SecondaryAddressType(locales::EN).fake()
}

pub fn city_prefix() -> String {
	f_address::CityPrefix(locales::EN).fake()
}

pub fn city_suffix() -> String {
	f_address::CitySuffix(locales::EN).fake()
}

pub fn city() -> String {
	f_address::CityName(locales::EN).fake()
}

pub fn state() -> String {
	f_address::StateName(locales::EN).fake()
}

pub fn state_abbr() -> String {
	f_address::StateAbbr(locales::EN).fake()
}

pub fn country() -> String {
	f_address::CountryName(locales::EN).fake()
}

pub fn country_code() -> String {
	f_address::CountryCode(locales::EN).fake()
}

pub fn zipcode() -> String {
	f_address::ZipCode(locales::EN).fake()
}

pub fn postalcode() -> String {
	f_address::PostCode(locales::EN).fake()
}

pub fn timezone() -> String {
	f_address::TimeZone(locales::EN).fake()
}

pub fn latitude() -> String {
	f_address::Latitude(locales::EN).fake()
}

pub fn longitude() -> String {
	f_address::Longitude(locales::EN).fake()
}

pub fn geohash(percision: u8) -> String {
	f_address::Geohash(locales::EN, percision).fake()
}
