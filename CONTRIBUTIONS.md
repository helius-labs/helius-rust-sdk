# Contribution Guide
## Style Guide
Code should adhere to the following principles:
- Use underscores, not dashes in filenames
- Do not use the name `main.rs` for a file
- Add tests for new features
- Follow Rust conventions and be consistent with existing code
- When in doubt, run `cargo fmt`

## Pull Requests
Pull Requests are the best way to propose changes to the SDK. We actively welcome any and all pull requests! To do so,
- Fork the repo and create your branch from `dev`
- Add tests if you've introduced new functionality that should be tested
- Update the relevant documentation if new functionality is introduced, or current functionality is altered
- Ensure the test suite passes (`cargo test`)
- Make the pull request!

Give pull requests a descriptive title. Examples of a good pull request title include:
- fix(enhanced_transactions): Issue with URL Format
- feat(zk_api): Add Get Private Balance 
- docs(webhooks): Add New Section on Deleting Webhooks

Examples of bad pull requests include:
- fix #76129
- update docs
- fix bugs

If there is a related issue, please reference it in the pull request's text.

## License
By contributing, you agree that your contributions will be licensed under its MIT License. Thus, when you submit code changes, your submissions are understood to be under the [following license](https://github.com/helius-labs/helius-rust-sdk/blob/dev/LICENSE)