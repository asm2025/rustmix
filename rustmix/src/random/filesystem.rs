use fake::{
    faker::filesystem::raw as f_filesystem,
    locales, Fake
};

pub fn dir_path() -> String {
	f_filesystem::DirPath(locales::EN).fake()
}

pub fn file_path() -> String {
	f_filesystem::FilePath(locales::EN).fake()
}

pub fn file_name() -> String {
	f_filesystem::FileName(locales::EN).fake()
}

pub fn file_extension() -> String {
	f_filesystem::FileExtension(locales::EN).fake()
}
