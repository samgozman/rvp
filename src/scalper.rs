use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::get;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::{Number, Value};

/// Parsed key-value structure
#[derive(Serialize, Clone)]
pub struct ParsedValue {
    pub name: String,
    pub value: Value,
}

/// It fetches the HTML from the given URL, parses it into a DOM, and then uses the given CSS selector
/// to extract the text from the first matching element
///
/// Arguments:
///
/// - `selector`: The CSS selector (full path from root) to use to grab the value.
/// - `from`: The URL to fetch the HTML from
///
/// Returns:
///
/// A [`Result<String>`]
pub async fn grab_one(selector: &str, from: &str) -> Result<String> {
    let document = fetch_html(from).await?;
    let selector = Selector::parse(selector).unwrap();
    parse_value(&document, &selector)
}

/// It takes a list of selectors and a URL, fetches the HTML from the URL, and then parses the HTML
/// using the selectors
///
/// Arguments:
///
/// - `selectors`: A vector of CSS selectors.
/// - `from`: The URL to fetch the HTML from.
///
/// Returns:
///
/// A vector of parsed values [Result<Vec<ParsedValue>>].
pub async fn grab(
    selectors: Vec<crate::structure::Selector>,
    from: String,
) -> Result<Vec<ParsedValue>> {
    let document = fetch_html(&from).await?;
    let mut values = Vec::new();

    for selector in selectors.iter() {
        let parsed = Selector::parse(&selector.path).unwrap();
        let value = parse_value(&document, &parsed)?;
        let value = match selector.parsed_type {
            crate::structure::SelectorType::String => Value::String(value),
            crate::structure::SelectorType::Number => {
                let number = any_string_to_number(&value);
                Value::Number(
                    Number::from_f64(number)
                        .expect(format!("failed to parse number for \"{}\"", &selector.name).as_str()),
                )
            }
        };
        values.push(ParsedValue {
            name: selector.name.clone(),
            value,
        });
    }

    Ok(values)
}

/// It fetches the HTML document at the given URL, parses it, and returns the result
///
/// Arguments:
///
/// - `url`: &str - The URL to fetch the HTML from.
///
/// Returns:
///
/// A [`Result<Html>`]
async fn fetch_html(url: &str) -> Result<Html> {
    let resp = match get(url).await {
        Ok(resp) => resp,
        Err(err) => return Err(anyhow!(err)),
    };
    let text = match resp.text().await {
        Ok(text) => text,
        Err(err) => return Err(anyhow!("failed to parse HTML document:\n{}", err)),
    };
    Ok(Html::parse_document(&text))
}

/// Parses the HTML document and returns the text of the first element that matches the selector.
///
/// Arguments:
///
/// - `document`: The HTML document we're parsing.
/// - `selector`: The CSS selector (full path from root) to use to find the element.
///
/// Returns:
///
/// A [String]
fn parse_value(document: &Html, selector: &Selector) -> Result<String> {
    let element = match document.select(selector).next() {
        Some(element) => element,
        // No need to panic if the selector doesn't match anything. Just return an empty string.
        // Let user decide what to do with it.
        None => return Ok("".to_string()),
    };

    Ok(element.text().collect::<Vec<_>>().join(" "))
}

/// Converts a complex string to a number
fn any_string_to_number(str: &str) -> f64 {
    let value = str.to_lowercase();
    let value = match str.contains(',') {
        // to avoid confusion between 1.000,00 and 1000.00
        true => value.replace('.', ""),
        false => value,
    };
    let value = value.replace(',', ".");

    // Remove all non-numeric characters except the dot and k/m/b
    let re = Regex::new(r"[^\r\n0-9.kmb]").unwrap();
    let value = re.replace_all(&value, "");

    // If string ends with "k", "m" or "b"
    let multiplier = match &value {
        v if v.ends_with('k') => 1000.0,
        v if v.ends_with('m') => 1000000.0,
        v if v.ends_with('b') => 1000000000.0,
        _ => 1.0,
    };

    // Remove the "k", "m" or "b" from the string
    let re = Regex::new(r"[^\r\n0-9.]").unwrap();
    let value = re.replace_all(&value, "");

    // Convert to float
    let num = value.parse::<f64>().unwrap_or(f64::NAN);
    num * multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_html() -> Result<()> {
        let document = fetch_html("http://example.com").await?;
        assert!(document
            .select(&Selector::parse("body").unwrap())
            .next()
            .is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_html_with_invalid_url() -> Result<()> {
        fetch_html("invalid-url")
            .await
            .expect_err("should fail with invalid URL!");
        Ok(())
    }

    #[tokio::test]
    async fn test_grab_one() -> Result<()> {
        let value = grab_one("body > div > h1", "http://example.com").await?;
        assert_eq!(value, "Example Domain");
        Ok(())
    }

    #[tokio::test]
    async fn test_grab() -> Result<()> {
        let selectors = vec![crate::structure::Selector {
            name: "title".to_string(),
            path: "body > div > h1".to_string(),
            parsed_type: crate::structure::SelectorType::String,
        }];
        let values = grab(selectors, "http://example.com".to_string()).await?;
        assert_eq!(values.len(), 1);
        assert_eq!(&values[0].name, "title");
        match &values[0].value {
            Value::String(value) => assert_eq!(value, "Example Domain"),
            _ => panic!("value should be a string!"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_grab_with_invalid_url() -> Result<()> {
        let selectors = vec![crate::structure::Selector {
            name: "title".to_string(),
            path: "body > div > h1".to_string(),
            parsed_type: crate::structure::SelectorType::String,
        }];
        if grab(selectors, "invalid-url".to_string()).await.is_ok() {
            panic!("should fail with invalid URL!");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_grab_with_invalid_selector() -> Result<()> {
        let selectors = vec![crate::structure::Selector {
            name: "title".to_string(),
            path: "body > div > h2".to_string(),
            parsed_type: crate::structure::SelectorType::String,
        }];
        if grab(selectors, "http://example.com".to_string())
            .await
            .is_err()
        {
            panic!("should not fail with invalid selector and return an empty string!");
        }
        Ok(())
    }

    #[test]
    fn test_parse_value() -> Result<()> {
        let document = Html::parse_document("<html><body><h1>Example</h1></body></html>");
        let selector = Selector::parse("h1").unwrap();
        let value = parse_value(&document, &selector)?;
        assert_eq!(value, "Example");
        Ok(())
    }

    #[test]
    fn test_parse_value_with_invalid_selector() -> Result<()> {
        let document = Html::parse_document("<html><body><h1>Example</h1></body></html>");
        let selector = Selector::parse("h2").unwrap();
        parse_value(&document, &selector).expect("should not fail with invalid selector!");
        Ok(())
    }

    #[test]
    fn test_any_string_to_number() {
        let value = any_string_to_number("1.234,56");
        assert_eq!(value, 1234.56);

        let value = any_string_to_number("100_000,5");
        assert_eq!(value, 100_000.5);

        let value = any_string_to_number("100 000 $");
        assert_eq!(value, 100_000.0);

        let value = any_string_to_number("1.5k$");
        assert_eq!(value, 1500.0);

        let value = any_string_to_number("1.5m ¢");
        assert_eq!(value, 1_500_000.0);

        let value = any_string_to_number("1.5b CAD$");
        assert_eq!(value, 1_500_000_000.0);

        let value = any_string_to_number("Not a Number");
        assert!(value.is_nan());
    }
}
