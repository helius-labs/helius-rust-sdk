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
- [Sign your commits](#signing-your-commits-required) — **required**; unsigned commits will be rejected
- Make the pull request!

### Signing Your Commits (Required)

**All commits must be signed and verified.** Pull requests that contain unsigned or unverified commits will fail CI and cannot be merged.

A "verified" commit is one GitHub can cryptographically tie to a registered signing key (GPG, SSH, or S/MIME). It shows a green **Verified** badge in the GitHub UI.

#### One-time setup

Follow GitHub's official guide to generate a signing key and configure Git to sign your commits:

**https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits**

The short version:

1. Generate or choose a signing key (GPG or SSH) — see the guide above.
2. Add the **public** key to your GitHub account under **Settings → SSH and GPG keys**.
3. Tell Git to use it and sign every commit automatically:

   ```bash
   # SSH signing (simplest if you already have an SSH key on GitHub)
   git config --global gpg.format ssh
   git config --global user.signingkey ~/.ssh/id_ed25519.pub
   git config --global commit.gpgsign true

   # — or — GPG signing
   git config --global user.signingkey <YOUR_KEY_ID>
   git config --global commit.gpgsign true
   ```

   Use `--global` to sign across all repos, or drop it to configure just this repo.

4. Confirm the email on your signing key matches a verified email on your GitHub account, otherwise commits show as **Unverified**.

#### Verifying it works

After committing, check the signature locally:

```bash
git log --show-signature -1
```

Once pushed, the commit should display a **Verified** badge on GitHub. If you have existing unsigned commits on a branch, you can re-sign them with:

```bash
git rebase --exec 'git commit --amend --no-edit -S' -i dev
```

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