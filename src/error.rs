//! Error handling helpers and primitives.

use anyhow::Error;

/// Chain creates a string from an error stack.
pub fn chain(res: Error) -> String {
    res.chain()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    #[test]
    fn chain() {
        let first = anyhow!("error 1");
        let second = anyhow!("error 2");

        let res = super::chain(first.context(second));

        assert_eq!(res, "error 2: error 1");
    }
}
