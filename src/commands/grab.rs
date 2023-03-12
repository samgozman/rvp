use anyhow::Result;
use clap::Parser;

use validator::Validate;

use crate::scalper;

/// Simply grab one value from a web page.
#[derive(Parser, Validate)]
pub struct Args {
    /// Selector path to grab from the page.
    ///
    /// You can get it from the browser's developer tools
    /// by right clicking on the HTML element and selecting "Copy > Selector Path" (or similar).
    ///
    /// Example: `-s="#search > div"`
    #[arg(short, long, value_name = "PATH")]
    #[validate(length(min = 1, message = "should not be empty!"))]
    selector: String,

    /// URL to web page to grab from.
    ///
    /// Example: `-f="https://example.com"`
    #[arg(short, long, value_name = "URL")]
    #[validate(url(message = "must be a valid URL!"))]
    from: String,
}

pub async fn command(args: Args) -> Result<()> {
    args.validate()?;

    let value = scalper::grab_one(&args.selector, &args.from).await?;

    println!("{}", value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command() -> Result<()> {
        let args = Args {
            selector: "body > div > h1".to_string(),
            from: "http://example.com".to_string(),
        };
        command(args).await
    }

    #[tokio::test]
    async fn test_command_with_invalid_url() -> Result<()> {
        let args = Args {
            selector: "#search > div".to_string(),
            from: "invalid-url".to_string(),
        };
        command(args)
            .await
            .expect_err("should fail with invalid URL!");
        Ok(())
    }

    #[tokio::test]
    async fn test_command_with_empty_selector() -> Result<()> {
        let args = Args {
            selector: "".to_string(),
            from: "http://example.com".to_string(),
        };
        command(args)
            .await
            .expect_err("should fail with empty selector!");
        Ok(())
    }
}
