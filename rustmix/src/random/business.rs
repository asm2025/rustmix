use fake::{
    faker::{
        administrative::raw as f_administrative, company::raw as f_company,
        creditcard::raw as f_creditcard, currency::raw as f_currency, finance::raw as f_finance,
        job::raw as f_job, phone_number::raw as f_phone_number,
    },
    locales, Fake,
};

pub fn insurance_code() -> String {
    f_administrative::HealthInsuranceCode(locales::FR_FR).fake()
}

pub fn company_name() -> String {
    f_company::CompanyName(locales::EN).fake()
}

pub fn company_suffix() -> String {
    f_company::CompanySuffix(locales::EN).fake()
}

pub fn industry() -> String {
    f_company::Industry(locales::EN).fake()
}

pub fn catch_phase() -> String {
    f_company::CatchPhrase(locales::EN).fake()
}

pub fn buzzword() -> String {
    f_company::Buzzword(locales::EN).fake()
}

pub fn buzzword_mid() -> String {
    f_company::BuzzwordMiddle(locales::EN).fake()
}

pub fn buzzword_tail() -> String {
    f_company::BuzzwordTail(locales::EN).fake()
}

pub fn bs() -> String {
    f_company::Bs(locales::EN).fake()
}

pub fn bs_adj() -> String {
    f_company::BsAdj(locales::EN).fake()
}

pub fn bs_noun() -> String {
    f_company::BsNoun(locales::EN).fake()
}

pub fn bs_verb() -> String {
    f_company::BsVerb(locales::EN).fake()
}

pub fn profession() -> String {
    f_company::Profession(locales::EN).fake()
}

pub fn credit_card() -> String {
    f_creditcard::CreditCardNumber(locales::EN).fake()
}

pub fn currency_code() -> String {
    f_currency::CurrencyCode(locales::EN).fake()
}

pub fn currency() -> String {
    f_currency::CurrencyName(locales::EN).fake()
}

pub fn currency_symbol() -> String {
    f_currency::CurrencySymbol(locales::EN).fake()
}

pub fn bic() -> String {
    f_finance::Bic(locales::EN).fake()
}

pub fn isin() -> String {
    f_finance::Isin(locales::EN).fake()
}

pub fn seniority() -> String {
    f_job::Seniority(locales::EN).fake()
}

pub fn job_field() -> String {
    f_job::Field(locales::EN).fake()
}

pub fn job_position() -> String {
    f_job::Position(locales::EN).fake()
}

pub fn job_title() -> String {
    f_job::Title(locales::EN).fake()
}

pub fn phone_number() -> String {
    f_phone_number::PhoneNumber(locales::EN).fake()
}

pub fn cell_number() -> String {
    f_phone_number::CellNumber(locales::EN).fake()
}
