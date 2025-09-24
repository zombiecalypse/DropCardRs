// --- HMR and Live Reload Cleanup ---
// We store animation frame IDs and event handlers on the global window object.
// This allows us to cancel them when Webpack's dev server re-runs the script,
// preventing duplicate game loops and event listeners.
if (window.dropCardCleanup) {
    window.dropCardCleanup();
}

window.dropCardCleanup = function() {
    if (window.dropCardAnimationId) {
        cancelAnimationFrame(window.dropCardAnimationId);
        window.dropCardAnimationId = null;
    }
    if (window.dropCardAnswerHandler) {
        const answerInput = document.getElementById('answer-input');
        if (answerInput) {
            answerInput.removeEventListener('keydown', window.dropCardAnswerHandler);
        }
        window.dropCardAnswerHandler = null;
    }
    if (window.dropCardTabHandler) {
        document.removeEventListener('keydown', window.dropCardTabHandler);
        window.dropCardTabHandler = null;
    }
    if (window.dropCardVisibilityHandler) {
        document.removeEventListener('visibilitychange', window.dropCardVisibilityHandler);
        window.dropCardVisibilityHandler = null;
    }
    // Clear the board to remove old cards and pause screen
    const gameBoard = document.getElementById('game-board');
    if (gameBoard) {
        const cardsContainer = document.getElementById('cards-container');
        if (cardsContainer) cardsContainer.innerHTML = '';
        const pauseScreen = document.getElementById('pause-screen');
        if (pauseScreen) pauseScreen.remove();
    }
};

// Run cleanup immediately to clear artifacts from any previous script execution
window.dropCardCleanup();
// ------------------------------------

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
    const gameBoard = document.getElementById('game-board');
    const cardsContainer = document.getElementById('cards-container');
    const scoreElement = document.getElementById('score');
    const healthElement = document.getElementById('health');
    const gameOverScreen = document.getElementById('game-over-screen');
    const answerInput = document.getElementById('answer-input');

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

    window.dropCardAnswerHandler = (event) => {
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
                console.log(`Enter pressed. Answer: "${answer}"`);
                if (answer) {
                    const correctly_answered = game.submit_answer(answer);
                    console.log(`submit_answer returned: ${correctly_answered}`);
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
    answerInput.addEventListener('keydown', window.dropCardAnswerHandler);

    window.dropCardTabHandler = (event) => {
        if (event.key === 'Tab' && !game.is_game_over()) {
            event.preventDefault();
            game.pause();
        }
    };
    document.addEventListener('keydown', window.dropCardTabHandler);

    window.dropCardVisibilityHandler = () => {
        if (document.hidden && !game.is_game_over()) {
            game.pause();
        }
    };
    document.addEventListener('visibilitychange', window.dropCardVisibilityHandler);

    let lastTime = 0;
    let lastLogTime = 0;
    function gameLoop(timestamp) {
        const deltaTime = (timestamp - lastTime) / 1000; // in seconds
        lastTime = timestamp;

        game.tick(deltaTime || 0);

        render(timestamp);

        window.dropCardAnimationId = requestAnimationFrame(gameLoop);
    }

    function render(timestamp) {
        cardsContainer.innerHTML = '';
        const cards = game.get_cards();

        if (timestamp - lastLogTime > 1000) {
            const card_fronts = cards.map(c => c.front).join(', ');
            console.log(`Rendering cards: [${card_fronts}]`);
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

    window.dropCardAnimationId = requestAnimationFrame(gameLoop);
}).catch(console.error);
