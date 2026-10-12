# Releasing to PyPI

Prerequisites: a PyPI account with 2FA, and ownership of the `binix` project name.

1. Bump `__version__` in `src/binix/__init__.py` (it is the single source of truth).
2. Update the status checklist in `README.md` if needed.
3. Make sure CI is green on `main`.
4. Build and check:

   ```sh
   python -m venv .venv && . .venv/bin/activate
   pip install -U build twine
   python -m build
   twine check dist/*
   ```

5. Rehearse on TestPyPI first:

   ```sh
   twine upload --repository testpypi dist/*
   pip install --index-url https://test.pypi.org/simple/ binix
   binix --version
   ```

6. Publish for real:

   ```sh
   twine upload dist/*
   ```

   Use a project-scoped API token (never your password), and never commit it.

7. Tag the release: `git tag v<version> && git push origin v<version>`.

A published version can be yanked but never re-uploaded, so always rehearse on TestPyPI.
