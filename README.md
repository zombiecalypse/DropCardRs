# Flashcards Game

A timed flashcard game built with Rust and WebAssembly.

This project is a simple browser-based game where flashcards fall from the top of the screen. The player must type the answer to the card's front before it reaches the bottom and flips over.

## Features

-   **Progressive Difficulty:** The game starts easy and gets harder as your score increases. Card speed, spawn rate, and the number of simultaneous cards all increase over time.
-   **Dynamic Card Unlocking:** Start with a small set of cards and unlock more as you score points.
-   **Shuffled Deck:** Cards are drawn from a shuffled deck to ensure all unlocked cards are practiced equally.
-   **Multiple Correct Answers:** Some cards accept multiple correct translations (e.g., "Thank you" and "Thanks").
-   **Game Pausing:** The game automatically pauses if the browser tab loses focus and can be manually paused with the `Tab` key.

## Game Modes

The game supports different modes for practicing, controlled via URL parameters.

-   **Normal Mode (Default):** Practice translating from Welsh to English.
    -   `http://localhost:8080`
-   **Reverse Mode:** Practice translating from English to Welsh.
    -   `http://localhost:8080/?mode=reverse`
-   **Both Mode:** Practice a random mix of both Welsh-to-English and English-to-Welsh.
    -   `http://localhost:8080/?mode=both`
-   **Debug Mode:** Displays a side panel with all currently unlocked cards. This can be combined with other modes.
    -   `http://localhost:8080/?debug=true`
    -   `http://localhost:8080/?mode=reverse&debug=true`

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
    The game will be available at `http://localhost:8080`. See the "Game Modes" section for URL parameters to change the game mode.

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

## Why?

This is a comparison to https://github.com/zombiecalypse/DropCard, which
creates the same game in pure javascript. Both were coded by AI, but presumably
pure javascript has a lot more online discussions that the model could have
learned from. 
