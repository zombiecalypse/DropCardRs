# Flashcards Game

A timed flashcard game built with Rust and WebAssembly.

This project is a simple browser-based game where flashcards fall from the top of the screen. The player must type the answer to the card's front before it reaches the bottom and flips over.

## Prerequisites

Before you begin, ensure you have the following installed:

-   [Rust](https://www.rust-lang.org/tools/install)
-   [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
-   [Node.js and npm](https://nodejs.org/en/download/)

## Building and Running

1.  **Build the WebAssembly package:**
    ```bash
    wasm-pack build
    ```

2.  **Install JavaScript dependencies:**
    ```bash
    npm install
    ```

3.  **Start the development server:**
    ```bash
    npm run serve
    ```
    The game will be available at `http://localhost:8080`.

## Running Tests

To run the test suite for the Rust game logic, you'll need a webdriver installed (e.g., `geckodriver` for Firefox).

```bash
wasm-pack test --headless --firefox
```

## Deployment

This project is configured for automated deployment to GitHub Pages. When changes are pushed to the `main` branch, a GitHub Action will build the project and deploy it.

The game will be available at [https://zombiecalypse.github.io/DropCardRs/](https://zombiecalypse.github.io/DropCardRs/).

To deploy manually, you can run:
```bash
npm run deploy
```
This will build the project and push the `dist` directory to the `gh-pages` branch. You will need to configure your repository on GitHub to serve from the `gh-pages` branch.
