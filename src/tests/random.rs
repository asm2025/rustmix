use chrono::{Duration, Utc};
use rustmix::random;

pub fn test_random() {
    println!("Testing random...");
    println!("string: {}", random::string(10));
    println!("alphanum: {}", random::alphanum_str(10));
    println!("boolean: {}", random::boolean());
    println!("float: {}", random::float());
    println!("numeric: {}", random::numeric(1..10));
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

    println!("dir_path: {}", random::filesystem::dir_path());
    println!("file_path: {}", random::filesystem::file_path());
    println!("file_name: {}", random::filesystem::file_name());
    println!("file_extension: {}", random::filesystem::file_extension());

    println!("status_code: {}", random::internet::status_code());
    println!(
        "valid_status_code: {}",
        random::internet::valid_status_code()
    );
    println!("mime_type: {}", random::internet::mime_type());
    println!("free_email: {}", random::internet::free_email());
    println!("safe_email: {}", random::internet::safe_email());
    println!(
        "free_email_provider: {}",
        random::internet::free_email_provider()
    );
    println!("domain_suffix: {}", random::internet::domain_suffix());
    println!("username: {}", random::internet::username());
    println!("password: {}", random::internet::password(8..64));
    println!("ipv4: {}", random::internet::ipv4());
    println!("ipv6: {}", random::internet::ipv6());
    println!("ip: {}", random::internet::ip());
    println!("mac_address: {}", random::internet::mac_address());
    println!("user_agent: {}", random::internet::user_agent());
    println!(
        "chrome_user_agent: {}",
        random::internet::user_agent().chrome()
    );
    println!(
        "firefox_user_agent: {}",
        random::internet::user_agent().firefox()
    );
    println!(
        "safari_user_agent: {}",
        random::internet::user_agent().safari()
    );
    println!(
        "desktop_user_agent: {}",
        random::internet::user_agent().desktop()
    );
    println!(
        "phone_user_agent: {}",
        random::internet::user_agent().phone()
    );

    println!("word: {}", random::lorem::word());
    println!("words: {}", random::lorem::words(1..10).join(", "));
    println!("sentence: {}", random::lorem::sentence(1..10));
    println!("sentences: {}", random::lorem::sentences(1..10).join("\n"));
    println!("paragraph: {}", random::lorem::paragraph(1..10));
    println!(
        "paragraphs: {}",
        random::lorem::paragraphs(1..10).join("\n")
    );

    println!("name: {}", random::person::name());
    println!("first_name: {}", random::person::first_name());
    println!("last_name: {}", random::person::last_name());
    println!("suffix: {}", random::person::suffix());
    println!("title: {}", random::person::title());
}
