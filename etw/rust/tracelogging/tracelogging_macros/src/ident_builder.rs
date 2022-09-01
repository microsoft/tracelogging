use std::fmt::Write;

pub struct IdentBuilder {
    ident: String,
    base_len: usize,
}

impl IdentBuilder {
    pub fn new(base_name: &str) -> IdentBuilder {
        let mut builder = Self {
            ident: String::with_capacity(base_name.len() + 4),
            base_len: base_name.len(),
        };

        builder.ident.push_str(base_name);

        return builder;
    }

    pub fn current(&self) -> &str {
        return &self.ident;
    }

    pub fn set_suffix(&mut self, suffix: usize) -> &str {
        self.ident.truncate(self.base_len);
        write!(self.ident, "{}", suffix).unwrap();
        return &self.ident;
    }

    pub fn clear_suffix(&mut self) -> &str {
        self.ident.truncate(self.base_len);
        return &self.ident;
    }
}
