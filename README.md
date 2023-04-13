# 🦀 RVP - Remote Value Parser

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/samgozman/rvp/cd.yml)
![GitHub](https://img.shields.io/github/license/samgozman/rvp)

RVP is a CLI tool written in Rust for parsing string values from static web pages. It can be used to extract information from financial sites, weather forecasts, and other types of web pages. With RVP, you can create configurations for each site to parse and retrieve information as a table or as JSON.

RVP also can parse complex numbers from web pages, including numbers with suffixes such as "k" or with underscores, such as "100_000.5".

## Features

* Parse string values from static web pages
* Create configurations for each site to parse via the CLI interface
* Retrieve information as a table or as JSON
* Parse complex numbers from web pages
* Save config files in TOML or JSON format
* Parse multiple values from multiple sites at once

## Example

🔺 // Place for gif with example

🔺 // Links to example config files

## Installation

RVP currently supports Intel Macs, M1 ARM Macs, and Linux. The tool has been tested on these platforms and is expected to work on other Unix-like systems as well. If you encounter any issues running RVP on your system, please let me know by creating an issue on the GitHub repository.

🔺 // Add manual installation instructions for macOS and Linux

## Usage

RVP can be used in two modes: simple mode and complex mode.

### Simple usage

In simple mode, you can use RVP to retrieve a single value from a single site. Just grab one value from one site:

```bash
rvp grab --selector="h1" --from="http://example.com"
```

Output: `Example Domain`

### Complex usage

In complex mode, you can create configuration files for each site that you want to parse. The configuration files specify the CSS selectors for the values you want to extract from the web page. You can then use RVP to parse multiple values from multiple sources using the configuration files.

#### Example 1: Parse stock information

Example config file: [stock.toml](examples/stock.toml)

For example, you can run the following command to parse the stock information from multiple sites:

```bash
rvp batch --path ./stock.toml --one-param AAPL --json
```

<details>

  <summary>Output</summary>
  
  ```json
    [
      {
        "name": "Name",
        "value": "Apple Inc."
      },
      {
        "name": "Market Cap",
        "value": "2519.25B"
      },
      {
        "name": "Price ($)",
        "value": 160.1
      },
      {
        "name": "Dividend ($)",
        "value": 0.92
      },
      {
        "name": "P/E",
        "value": 27.2
      },
      {
        "name": "% of Float Shorted",
        "value": 0.71
      },
      {
        "name": "Industry",
        "value": "Computers/Consumer Electronics"
      },
      {
        "name": "Sector",
        "value": "Technology"
      },
      {
        "name": "Put/Call Vol Ratio",
        "value": 0.77
      },
      {
        "name": "Put/Call OI Ratio ",
        "value": 1.01
      }
    ]
  ```

</details>

> `--one-param` option can be specified for each site in the config file. It simply replaces the `%%` placeholder in the URL. With this option, you can specify a **single parameter** that will be passed for all resources with the `%%` placeholder in the URL.

#### Example 2: Get weather forecasts for multiple cities

🔺 // Link to the example config file

You can run the following command to parse the weather forecast for multiple cities:

```bash
rvp batch --path ./weather.toml --params "new-york/new-york" "california/los-angeles" "illinois/chicago" --json
```

> `--params` option can be specified for each site in the config file. It simply replaces the `%%` placeholder in the URL. If you have **multiple resources** to parse, you can specify them as a **space-separated list**.

RVP's batch mode allows you to retrieve information from multiple sources and multiple values at once, making it a powerful tool for web scraping and data extraction.

## Create config file

To create a new configuration file for a website, you can use the `new` command followed by the `--name` flag to specify the name of the configuration file:

For example, to create a configuration file for the weather forecast, run the following command:

```bash
rvp new --name weather
```

It will ask you in which format you want to save the configuration file. You can choose between TOML and JSON. The configuration file will be saved in the current directory.

RVP will start a CLI dialog that guides you through the process of creating the configuration file. In the dialog, you will be prompted to add resources (websites) and selectors for the values you want to extract from each website.

If you want to add a variable to the URL, you can use the `%%` placeholder. For example, if you want to parse the weather forecast for different cities, you can use the `%%` placeholder in the URL and specify the city name as a parameter when running the `batch` command.

When adding selectors, you will need to provide a full CSS selector path for the value you want to extract. To find the CSS selector path in the Google Chrome browser, you can right-click on the element containing the value and select "Inspect". This will open the Chrome DevTools, and the corresponding HTML element will be highlighted in the Elements panel. You can then right-click on the highlighted element and select "Copy" > "Copy selector" to copy the full CSS selector path to the clipboard. You can then paste the selector into the CLI dialog when prompted.

By following the CLI dialog, you can create a new configuration file for any website you want to parse with RVP, making it easy to customize the tool for your specific needs.

## Contributing

Contributions to RVP are welcome! If you have a feature request or find a bug, please create an issue on the GitHub repository. Pull requests are also welcome.

## License

RVP is licensed under the MIT license. See the [LICENSE](LICENSE) file for more information.
