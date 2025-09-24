// --- Robust Cleanup for HMR and Dev Server double-load ---
// If a cleanup function from a previous run exists, it means we're in a
// hot-reload or double-load scenario. Run it to tear down the old instance.
if (window.flashCardGameCleanup) {
    console.log("Previous game instance found. Running cleanup.");
    window.flashCardGameCleanup();
}
// ---------------------------------------------------------

import('../pkg/flashcards.js').then(module => {
    const { Game } = module;

    const style = document.createElement('style');
    style.textContent = `
        .shake {
            animation: shake 0.5s;
        }
        @keyframes shake {
            10%, 90% { transform: translateX(-1px); }
            20%, 80% { transform: translateX(2px); }
            30%, 50%, 70% { transform: translateX(-4px); }
            40%, 60% { transform: translateX(4px); }
        }
    `;
    document.head.appendChild(style);

    const GAME_WIDTH = 600;
    const GAME_HEIGHT = 800;

    const game = Game.new(GAME_WIDTH, GAME_HEIGHT);
    const gameId = game.get_id();
    console.log(`[Game ${gameId}] Initialized.`);
    const gameBoard = document.getElementById('game-board');
    const cardsContainer = document.getElementById('cards-container');
    const scoreElement = document.getElementById('score');
    const healthElement = document.getElementById('health');
    const gameOverScreen = document.getElementById('game-over-screen');
    const answerInput = document.getElementById('answer-input');

    // --- State for cleanup ---
    let animationFrameId = null;
    let answerHandler = null;
    let tabKeyHandler = null;
    let visibilityChangeHandler = null;
    // ---

    const pauseScreen = document.createElement('div');
    pauseScreen.id = 'pause-screen';
    pauseScreen.className = 'overlay';
    pauseScreen.innerHTML = '<h1>Paused</h1><p>Press Enter to continue</p>';
    Object.assign(pauseScreen.style, {
        position: 'absolute',
        top: '0',
        left: '0',
        width: '100%',
        height: '100%',
        backgroundColor: 'rgba(0, 0, 0, 0.7)',
        color: 'white',
        zIndex: '20',
        display: 'none',
        justifyContent: 'center',
        alignItems: 'center',
        flexDirection: 'column',
    });
    gameBoard.appendChild(pauseScreen);

    answerHandler = (event) => {
        if (event.key === 'Enter') {
            event.preventDefault();
            if (game.is_game_over()) {
                game.restart();
                gameOverScreen.classList.add('hidden');
                pauseScreen.style.display = 'none';
                answerInput.focus();
            } else if (game.is_paused()) {
                game.resume();
            } else {
                const answer = answerInput.value;
                console.log(`[Game ${gameId}] Enter pressed. Answer: "${answer}"`);
                if (answer) {
                    const correctly_answered = game.submit_answer(answer);
                    console.log(`[Game ${gameId}] submit_answer returned: ${correctly_answered}`);
                    if (!correctly_answered) {
                        gameBoard.classList.add('shake');
                        setTimeout(() => {
                            gameBoard.classList.remove('shake');
                        }, 500);
                    }
                    answerInput.value = '';
                }
            }
        }
    };
    answerInput.addEventListener('keydown', answerHandler);

    tabKeyHandler = (event) => {
        if (event.key === 'Tab' && !game.is_game_over()) {
            event.preventDefault();
            game.pause();
        }
    };
    document.addEventListener('keydown', tabKeyHandler);

    visibilityChangeHandler = () => {
        if (document.hidden && !game.is_game_over()) {
            game.pause();
        }
    };
    document.addEventListener('visibilitychange', visibilityChangeHandler);

    let lastTime = 0;
    let lastLogTime = 0;
    function gameLoop(timestamp) {
        const deltaTime = (timestamp - lastTime) / 1000; // in seconds
        lastTime = timestamp;

        game.tick(deltaTime || 0);

        render(timestamp);

        animationFrameId = requestAnimationFrame(gameLoop);
    }

    function render(timestamp) {
        cardsContainer.innerHTML = '';
        const cards = game.get_cards();

        if (timestamp - lastLogTime > 1000) {
            const card_fronts = cards.map(c => c.front).join(', ');
            console.log(`[Game ${gameId}] Rendering cards: [${card_fronts}]`);
            lastLogTime = timestamp;
        }

        for (const card of cards) {
            const cardElement = document.createElement('div');
            cardElement.className = 'card';
            if (card.flipped) {
                cardElement.classList.add('flipped');
            }
            cardElement.style.left = `${card.x}px`;
            cardElement.style.top = `${card.y}px`;

            const front = document.createElement('div');
            front.className = 'front';
            front.textContent = card.front;

            const back = document.createElement('div');
            back.className = 'back';
            back.textContent = card.back;

            cardElement.appendChild(front);
            cardElement.appendChild(back);
            cardsContainer.appendChild(cardElement);
        }

        scoreElement.textContent = `Score: ${game.get_score()}`;

        // Render health
        const health = game.get_health();
        const maxHealth = game.get_max_health();
        let hearts = '';
        for (let i = 0; i < health; i++) {
            hearts += '❤️';
        }
        for (let i = 0; i < maxHealth - health; i++) {
            hearts += '🖤';
        }
        healthElement.innerHTML = hearts;

        // Game over
        if (game.is_game_over()) {
            gameOverScreen.classList.remove('hidden');
        }

        // Pause
        if (game.is_paused()) {
            pauseScreen.style.display = 'flex';
        } else {
            pauseScreen.style.display = 'none';
        }
    }

    animationFrameId = requestAnimationFrame(gameLoop);

    // Define and attach the cleanup function to the window object. This function
    // captures the current game's state and handlers in a closure, allowing
    // the *next* script execution to clean up this one.
    window.flashCardGameCleanup = () => {
        console.log(`[Game ${gameId}] Cleaning up instance.`);
        cancelAnimationFrame(animationFrameId);
        answerInput.removeEventListener('keydown', answerHandler);
        document.removeEventListener('keydown', tabKeyHandler);
        document.removeEventListener('visibilitychange', visibilityChangeHandler);

        // Also clear the board to prevent visual artifacts
        const pauseScreen = document.getElementById('pause-screen');
        if (pauseScreen) pauseScreen.remove();
        // Check for cardsContainer existence before manipulating
        if (cardsContainer) cardsContainer.innerHTML = '';
    };

    if (module.hot) {
        module.hot.dispose(() => {
            if (window.flashCardGameCleanup) {
                console.log(`[Game ${gameId}] HMR dispose. Cleaning up.`);
                window.flashCardGameCleanup();
            }
        });
    }
}).catch(console.error);
