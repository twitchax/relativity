# Contributing to relativity

Thank you for your interest in contributing to relativity!

## How to Contribute

1. **Fork the repository** and create your branch from `main`.
2. **Make your changes** following the coding style and guidelines below.
3. **Test your changes** by running `cargo make ci` (format check + clippy + tests).
4. **Format your code** with `cargo make fmt`.
5. **Submit a pull request** with a clear description of your changes.

## Coding Guidelines

- Follow the existing code style and conventions.
- Use `cargo make fmt` to format your code before committing.
- Ensure `cargo make clippy` passes with no warnings.
- Write clear, descriptive commit messages.
- Add tests for new functionality where applicable.
- Keep changes focused and avoid mixing unrelated changes in a single PR.

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed instructions on setting up your development environment, building the project, and running tests.

## Reporting Issues

If you find a bug or have a feature request, please open an issue on GitHub with:
- A clear, descriptive title
- Steps to reproduce (for bugs)
- Expected vs. actual behavior
- Any relevant logs or screenshots

## Code of Conduct

Be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## License

By contributing to relativity, you agree that your contributions will be licensed under the MIT License.
