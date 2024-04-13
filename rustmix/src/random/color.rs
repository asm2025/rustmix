use fake::{
    faker::color::raw as f_color,
    locales, Fake
};

pub fn name() -> String {
	f_color::Color(locales::EN).fake()
}

pub fn hex() -> String {
	f_color::HexColor(locales::EN).fake()
}

pub fn rgb() -> String {
	f_color::RgbColor(locales::EN).fake()
}

pub fn rgba() -> String {
	f_color::RgbaColor(locales::EN).fake()
}

pub fn hsl() -> String {
	f_color::HslColor(locales::EN).fake()
}

pub fn hsla() -> String {
	f_color::HslaColor(locales::EN).fake()
}
