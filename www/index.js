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

    answerInput.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
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
    });

    document.addEventListener('keydown', (event) => {
        if (event.key === 'Tab' && !game.is_game_over()) {
            event.preventDefault();
            game.pause();
        }
    });

    document.addEventListener('visibilitychange', () => {
        if (document.hidden && !game.is_game_over()) {
            game.pause();
        }
    });

    let lastTime = 0;
    function gameLoop(timestamp) {
        const deltaTime = (timestamp - lastTime) / 1000; // in seconds
        lastTime = timestamp;

        game.tick(deltaTime || 0);

        render();

        requestAnimationFrame(gameLoop);
    }

    function render() {
        cardsContainer.innerHTML = '';
        const cards = game.get_cards();

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

    requestAnimationFrame(gameLoop);
}).catch(console.error);
