# Helius SDK
An asynchronous Helius Rust SDK for building the future of Solana

## Documentation

## Installation

## Usage
The SDK needs to be configured with your account's API key, which can be found on the [Helius Developer Dashboard](https://dev.helius.xyz/dashboard/app)

## Error Handling

### Common Error Codes
You may encounter several error codes when working with the Helius Rust SDK. Below is a table detailing some of the common error codes along with additional information to aid with troubleshooting:

| Error Code | Error Message             | More Information                                                                           |
|------------|---------------------------|---------------------------------------------------------------------------------------------|
| 401        | Unauthorized              | This occurs when an invalid API key is provided or access is restricted |
| 429        | Too Many Requests         | This indicates that the user has exceeded the request limit in a given timeframe or is out of credits |
| 5XX        | Internal Server Error     | This is a generic error message for server-side issues. Please contact Helius support for assistance |

If you encounter any of these errors, refer to the Helius documentation for further guidance, or reach out to the Helius support team for more detailed assistance

## Using the Helius SDK
Our SDK is designed to provide a seamless developer experience when building on Solana. We've separated the core functionality into various segments:

DAS API

Mint API

Webhooks

Helper Methods