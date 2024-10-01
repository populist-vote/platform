use slugify::slugify;

pub struct PartySlugGenerator<'a> {
    pub name: &'a str,
}

impl<'a> PartySlugGenerator<'a> {
    pub fn new(name: &'a str) -> Self {
        PartySlugGenerator { name }
    }

    pub fn generate(&self) -> String {
        slugify!(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn party_slug() {
        let tests: Vec<(&'static str, &'static str)> = vec![("Bestest Party", "bestest-party")];

        for (input, expected) in tests {
            assert_eq!(PartySlugGenerator::new(input).generate(), expected);
        }
    }
}
