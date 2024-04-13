use fake::{
    faker::lorem::raw as f_lorem,
    locales, Fake
};
use std::ops::Range;

pub fn word() -> String {
	f_lorem::Word(locales::EN).fake()
}

pub fn words(count: Range<usize>) -> Vec<String> {
	f_lorem::Words(locales::EN, count).fake()
}

pub fn sentence(count: Range<usize>) -> String {
	f_lorem::Sentence(locales::EN, count).fake()
}

pub fn sentences(count: Range<usize>) -> Vec<String> {
	f_lorem::Sentences(locales::EN, count).fake()
}

pub fn paragraph(count: Range<usize>) -> String {
	f_lorem::Paragraph(locales::EN, count).fake()
}

pub fn paragraphs(count: Range<usize>) -> Vec<String> {
	f_lorem::Paragraphs(locales::EN, count).fake()
}
