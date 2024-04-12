pub const SPECIAL_CHARS: [char; 10] = ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'];

pub trait StringEx {
    fn trim(&self, ch: &char) -> &str;
    fn trim_start(&self, ch: &char) -> &str;
    fn trim_end(&self, ch: &char) -> &str;
    fn trim_many(&self, ch: &[char]) -> &str;
    fn trim_start_many(&self, ch: &[char]) -> &str;
    fn trim_end_many(&self, ch: &[char]) -> &str;
    fn prefix(&self, ch: char) -> String;
    fn suffix(&self, ch: char) -> String;
}

impl StringEx for str {
    fn trim(&self, ch: &char) -> &str {
        if self.is_empty() {
            return self;
        }

        let mut start = 0;
        let mut end = self.len();

        for (i, c) in self.chars().enumerate() {
            if c != *ch {
                start = i;
                break;
            }
        }

        for (i, c) in self.chars().rev().enumerate() {
            if c != *ch {
                end = self.len() - i;
                break;
            }
        }

        &self[start..end]
    }

    fn trim_start(&self, ch: &char) -> &str {
        if self.is_empty() {
            return self;
        }

        let mut start = 0;

        for (i, c) in self.chars().enumerate() {
            if c != *ch {
                start = i;
                break;
            }
        }

        &self[start..]
    }

    fn trim_end(&self, ch: &char) -> &str {
        if self.is_empty() {
            return self;
        }

        let mut end = self.len();

        for (i, c) in self.chars().rev().enumerate() {
            if c != *ch {
                end = self.len() - i;
                break;
            }
        }

        &self[..end]
    }

    fn trim_many(&self, ch: &[char]) -> &str {
        if self.is_empty() {
            return self;
        }

        let mut start = 0;
        let mut end = self.len();

        for (i, c) in self.chars().enumerate() {
            if !ch.contains(&c) {
                start = i;
                break;
            }
        }

        for (i, c) in self.chars().rev().enumerate() {
            if !ch.contains(&c) {
                end = self.len() - i;
                break;
            }
        }

        &self[start..end]
    }

    fn trim_start_many(&self, ch: &[char]) -> &str {
        if self.is_empty() {
            return self;
        }

        let mut start = 0;

        for (i, c) in self.chars().enumerate() {
            if !ch.contains(&c) {
                start = i;
                break;
            }
        }

        &self[start..]
    }

    fn trim_end_many(&self, ch: &[char]) -> &str {
        if self.is_empty() {
            return self;
        }

        let mut end = self.len();

        for (i, c) in self.chars().rev().enumerate() {
            if !ch.contains(&c) {
                end = self.len() - i;
                break;
            }
        }

        &self[..end]
    }

    fn prefix(&self, ch: char) -> String {
        if !self.starts_with(ch) {
            format!("{}{}", ch, self)
        } else {
            self.to_owned()
        }
    }

    fn suffix(&self, ch: char) -> String {
        if !self.ends_with(ch) {
            format!("{}{}", self, ch)
        } else {
            self.to_owned()
        }
    }
}
