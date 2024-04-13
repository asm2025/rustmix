use fake::{
    faker::name::raw as f_name,
    locales, Fake
};

pub fn first_name() -> String {
	f_name::FirstName(locales::EN).fake()
}

pub fn last_name() -> String {
	f_name::LastName(locales::EN).fake()
}

pub fn name() -> String {
	f_name::Name(locales::EN).fake()
}

pub fn title() -> String {
	f_name::Title(locales::EN).fake()
}

pub fn name_with_title() -> String {
	f_name::NameWithTitle(locales::EN).fake()
}

pub fn suffix() -> String {
	f_name::Suffix(locales::EN).fake()
}
