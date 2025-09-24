import('../pkg/flashcards.js').then(module => {
    const { Game } = module;

    const GAME_WIDTH = 600;
    const GAME_HEIGHT = 800;

    const game = Game.new(GAME_WIDTH, GAME_HEIGHT);
    const gameBoard = document.getElementById('game-board');
    const scoreElement = document.getElementById('score');
    const answerInput = document.getElementById('answer-input');

    answerInput.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            const answer = answerInput.value;
            if (answer) {
                if (game.submit_answer(answer)) {
                    answerInput.value = '';
                }
            }
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
        gameBoard.innerHTML = '';
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
            gameBoard.appendChild(cardElement);
        }

        scoreElement.textContent = `Score: ${game.get_score()}`;
    }

    requestAnimationFrame(gameLoop);
}).catch(console.error);
