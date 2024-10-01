use slugify::slugify;

pub struct PoliticianSlugGenerator<'a> {
    pub source: &'a str,
    pub name: &'a str,
}

impl<'a> PoliticianSlugGenerator<'a> {
    pub fn new(source: &'a str, name: &'a str) -> Self {
        PoliticianSlugGenerator { source, name }
    }

    pub fn generate(&self) -> String {
        slugify!(&format!("{} {}", self.source, self.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn party_slug() {
        let tests: Vec<((&'static str, &'static str), &'static str)> = vec![
            (("CO SOS", "John Smith"), "co-sos-john-smith"),
            (("MN CSV", "John Smith"), ("mn-csv-john-smith")),
        ];

        for (input, expected) in tests {
            assert_eq!(
                PoliticianSlugGenerator::new(input.0, input.1).generate(),
                expected
            );
        }
    }
}
