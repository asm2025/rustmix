use chrono::{Duration, Utc};
use rustmix::random;

pub fn test_random() {
    println!("Testing random...");
    println!("string: {}", random::string(10));
    println!("alphanum: {}", random::alphanum_str(10));
    println!("boolean: {}", random::boolean());
    println!("float: {}", random::float());
    println!("numeric: {}", random::numeric(1, 10));
    println!("uuid: {}", random::uuid());
    println!("uuid v3: {}", random::uuid_v(random::UuidVersion::V3));
    println!("uuid v5: {}", random::uuid_v(random::UuidVersion::V5));

    println!("building: {}", random::address::building());
    println!("street: {}", random::address::street());
    println!("street_suffix: {}", random::address::street_suffix());
    println!(
        "secondary_address: {}",
        random::address::secondary_address()
    );
    println!(
        "secondary_address_type: {}",
        random::address::secondary_address_type()
    );
    println!("city_prefix: {}", random::address::city_prefix());
    println!("city_suffix: {}", random::address::city_suffix());
    println!("city: {}", random::address::city());
    println!("state: {}", random::address::state());
    println!("state_abbr: {}", random::address::state_abbr());
    println!("country: {}", random::address::country());
    println!("country_code: {}", random::address::country_code());
    println!("zipcode: {}", random::address::zipcode());
    println!("postalcode: {}", random::address::postalcode());
    println!("timezone: {}", random::address::timezone());
    println!("latitude: {}", random::address::latitude());
    println!("longitude: {}", random::address::longitude());
    println!("geohash: {}", random::address::geohash(8));

    println!("license_number: {}", random::automotive::license_number());

    println!("isbn: {}", random::barcode::isbn());
    println!("isbn10: {}", random::barcode::isbn10());
    println!("isbn13: {}", random::barcode::isbn13());

    println!("insurance_code: {}", random::business::insurance_code());
    println!("company_name: {}", random::business::company_name());
    println!("company_suffix: {}", random::business::company_suffix());
    println!("industry: {}", random::business::industry());
    println!("catch_phase: {}", random::business::catch_phase());
    println!("bs: {}", random::business::bs());

    println!("color: {}", random::color::name());
    println!("hex: {}", random::color::hex());
    println!("rgb: {}", random::color::rgb());
    println!("rgba: {}", random::color::rgba());
    println!("hsl: {}", random::color::hsl());
    println!("hsla: {}", random::color::hsla());

    let date = Utc::now();
    let date2 = date.checked_add_signed(Duration::days(5)).unwrap();
    println!("naive: {:?}", random::datetime::naive());
    println!("str: {}", random::datetime::str());
    println!("before {:?}: {:?}", date, random::datetime::before(date));
    println!("after {:?}: {:?}", date, random::datetime::after(date));
    println!(
        "between {:?} and {:?}: {:?}",
        date,
        date2,
        random::datetime::between(date, date2)
    );
    println!("date: {:?}", random::datetime::date());
    println!("time: {:?}", random::datetime::time());
    println!("duration: {:?}", random::datetime::duration());

    println!("word: {}", random::lorem::word());
}
