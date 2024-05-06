# Contribution Guide
Welcome to the Helius Rust SDK! We value your contributions and want to make it as easy as possible for you to contribute. Here's how you can help make the SDK better

## Style Guide
To maintain high standards of quality and readability, code should adhere to the following principles:
- **Filenames**: Use underscores instead of dashes in filenames
- **Main Entry Point**: Avoid naming files as `main.rs`. Use descriptive names that reflect their functionality
- **Testing**: Add tests for any new features or functionality added
- **Formatting**: Follow Rust conventions. Be consistent with the existing codebase. When in doubt, run `cargo fmt`
- **Documentation**: Keep the documentation up to date. If you introduce new features or changes that affect how users interact with the SDK, update the documentation accordingly

## Pull Requests
Pull Requests are the best way to propose changes to the SDK. We actively welcome any and all pull requests! To do so,
- Fork the repo and create your branch from `dev`
- Add tests if you've introduced new functionality that should be tested
- Update the relevant documentation if new functionality is introduced, or current functionality is altered
- Ensure the test suite passes (`cargo test`)
- Make the pull request!

### Good Pull Request Titles
- fix(enhanced_transactions): Issue with URL Format
- feat(zk_api): Add Get Private Balance 
- docs(webhooks): Add New Section on Deleting Webhooks

### Bad Pull Request Titles
- fix #76129
- update docs
- fix bugs

### Related Issues
If there is a related issue, please reference it in the pull request's text.

## License
By contributing, you agree that your contributions will be licensed under its MIT License. Thus, when you submit code changes, your submissions are understood to be under the [following license](https://github.com/helius-labs/helius-rust-sdk/blob/dev/LICENSE)

## Thank You!
We deeply appreciate your effort in improving the Helius Rust SDK. Your contributions help make the SDK a valuable tool for everyone