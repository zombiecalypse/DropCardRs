if (window.isFlashCardGameRunning) {
    console.warn("Skipping duplicate game initialization.");
} else {
    window.isFlashCardGameRunning = true;

    import('../pkg/flashcards.js').then(module => {
    const { Game, GameMode } = module;

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
        .card.solved {
            animation: solved-animation 0.5s forwards;
        }
        @keyframes solved-animation {
            to {
                transform: scale(1.1);
                opacity: 0;
            }
        }
    `;
    document.head.appendChild(style);

    const GAME_WIDTH = 600;
    const GAME_HEIGHT = 800;

    const urlParams = new URLSearchParams(window.location.search);
    const mode_str = urlParams.get('mode');
    let mode = GameMode.Normal;
    if (mode_str === 'reverse') {
        mode = GameMode.Reverse;
    } else if (mode_str === 'both') {
        mode = GameMode.Both;
    }
    const isDebug = urlParams.get('debug') === 'true';
    
    const seed = Math.floor(Math.random() * 2**32);
    const game = Game.new(GAME_WIDTH, GAME_HEIGHT, seed, mode);
    const gameId = game.get_id();
    console.log(`[Game ${gameId}] Initialized.`);
    const gameBoard = document.getElementById('game-board');
    const cardsContainer = document.getElementById('cards-container');
    const scoreElement = document.getElementById('score');
    const healthElement = document.getElementById('health');
    const gameOverScreen = document.getElementById('game-over-screen');
    const answerInput = document.getElementById('answer-input');

    let debugPane = null;
    let cardElements = new Map();

    if (isDebug) {
        debugPane = document.createElement('div');
        debugPane.id = 'debug-pane';
        Object.assign(debugPane.style, {
            position: 'absolute',
            top: '10px',
            right: '10px',
            width: '250px',
            maxHeight: 'calc(100% - 20px)',
            overflowY: 'auto',
            backgroundColor: 'rgba(255, 255, 255, 0.9)',
            border: '1px solid #ccc',
            padding: '10px',
            fontFamily: 'monospace',
            fontSize: '12px',
            zIndex: '100',
        });
        document.body.appendChild(debugPane);
    }

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
                if (answer) {
                    if (!game.submit_answer(answer)) {
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
        const currentCards = game.get_cards();
        const currentCardIds = new Set(currentCards.map(c => c.id));

        // Animate and remove solved cards
        for (const [id, element] of cardElements.entries()) {
            if (!currentCardIds.has(id)) {
                element.classList.add('solved');
                element.addEventListener('animationend', () => {
                    element.remove();
                });
                cardElements.delete(id);
            }
        }

        if (isDebug && debugPane) {
            const unlockedCards = game.get_unlocked_cards();
            let content = '<h3>Unlocked Cards</h3><ul>';
            for (const card of unlockedCards) {
                content += `<li><b>${card.front}</b> - ${card.back}</li>`;
            }
            content += '</ul>';
            debugPane.innerHTML = content;
        }

        // Add or update cards on screen
        for (const card of currentCards) {
            let cardElement = cardElements.get(card.id);

            if (!cardElement) { // New card
                cardElement = document.createElement('div');
                cardElement.className = 'card';
                cardElements.set(card.id, cardElement);
                cardsContainer.appendChild(cardElement);

                const front = document.createElement('div');
                front.className = 'front';
                cardElement.appendChild(front);

                const back = document.createElement('div');
                back.className = 'back';
                cardElement.appendChild(back);
            }
            
            // Update common properties
            if (card.flipped) {
                cardElement.classList.add('flipped');
            } else {
                cardElement.classList.remove('flipped');
            }
            cardElement.style.left = `${card.x}px`;
            cardElement.style.top = `${card.y}px`;
            
            cardElement.querySelector('.front').textContent = card.front;
            cardElement.querySelector('.back').textContent = card.back;
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

    if (module.hot) {
        module.hot.dispose(() => {
            // A full reload is the most robust way to handle HMR
            window.location.reload();
        });
    }
    }).catch(console.error);
}
