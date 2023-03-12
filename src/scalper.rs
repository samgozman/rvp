use anyhow::{anyhow, Result};
use reqwest::get;
use scraper::{Html, Selector};

/// It fetches the HTML from the given URL, parses it into a DOM, and then uses the given CSS selector
/// to extract the text from the first matching element
///
/// Arguments:
///
/// * `selector`: The CSS selector (full path from root) to use to grab the value.
/// * `from`: The URL to fetch the HTML from
///
/// Returns:
///
/// A [`Result<String>`]
pub async fn grab_one(selector: &str, from: &str) -> Result<String> {
    let document = fetch_html(from).await?;
    let selector = Selector::parse(selector).unwrap();
    parse_value(&document, &selector)
}

/// It fetches the HTML document at the given URL, parses it, and returns the result
///
/// Arguments:
///
/// * `url`: &str - The URL to fetch the HTML from.
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
/// * `document`: The HTML document we're parsing.
/// * `selector`: The CSS selector (full path from root) to use to find the element.
///
/// Returns:
///
/// A [String]
fn parse_value(document: &Html, selector: &Selector) -> Result<String> {
    let element = match document.select(selector).next() {
        Some(element) => element,
        None => return Err(anyhow!("selector not found in the HTML document")),
    };

    Ok(element.text().collect::<Vec<_>>().join(" "))
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
    async fn test_parse_value() -> Result<()> {
        let document = Html::parse_document("<html><body><h1>Example</h1></body></html>");
        let selector = Selector::parse("h1").unwrap();
        let value = parse_value(&document, &selector)?;
        assert_eq!(value, "Example");
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_value_with_invalid_selector() -> Result<()> {
        let document = Html::parse_document("<html><body><h1>Example</h1></body></html>");
        let selector = Selector::parse("h2").unwrap();
        parse_value(&document, &selector).expect_err("should fail with invalid selector!");
        Ok(())
    }

    #[tokio::test]
    async fn test_grab_one() -> Result<()> {
        let value = grab_one("body > div > h1", "http://example.com").await?;
        assert_eq!(value, "Example Domain");
        Ok(())
    }
}
