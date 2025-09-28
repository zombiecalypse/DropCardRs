if (window.isFlashCardGameRunning) {
    console.warn("Skipping duplicate game initialization.");
} else {
    window.isFlashCardGameRunning = true;

    import('../pkg/flashcards.js').then(async (module) => {
    await module.default();
    const { Game, GameMode } = module;

    const startScreen = document.getElementById('start-screen');
    const startDefaultBtn = document.getElementById('start-default-btn');
    const ankiImportInput = document.getElementById('anki-import-input');
    const gameContainer = document.getElementById('game-container');
    const deckConfigScreen = document.getElementById('deck-config-screen');
    const cardListContainer = document.getElementById('card-list-container');
    const startConfiguredGameBtn = document.getElementById('start-configured-game-btn');
    let loadedDeck = [];

    function initializeAndRunGame(configuredDeck) {
        gameContainer.classList.remove('hidden');
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

    const gameBoard = document.getElementById('game-board');
    const GAME_WIDTH = gameBoard.clientWidth;
    const GAME_HEIGHT = gameBoard.clientHeight;

    const urlParams = new URLSearchParams(window.location.search);
    const mode_str = urlParams.get('mode');
    let mode = GameMode.Normal;
    if (mode_str === 'reverse') {
        mode = GameMode.Reverse;
    } else if (mode_str === 'both') {
        mode = GameMode.Both;
    }
    const isDebug = urlParams.get('debug') === 'true';

    const isMobile = window.innerWidth <= 600;
    const speedMultiplier = isMobile ? 0.75 : 1.0;
    
    const seed = BigInt(Math.floor(Math.random() * 2**32));
    let game;
    try {
        game = Game.new(GAME_WIDTH, GAME_HEIGHT, seed, mode, speedMultiplier, configuredDeck);
    } catch (e) {
        alert(`Error initializing game: ${e}`);
        startScreen.classList.remove('hidden');
        gameContainer.classList.add('hidden');
        ankiImportInput.value = '';
        return;
    }
    const gameId = game.get_id();
    console.log(`[Game ${gameId}] Initialized.`);
    const cardsContainer = document.getElementById('cards-container');
    const scoreElement = document.getElementById('score');
    const healthElement = document.getElementById('health');
    const gameOverScreen = document.getElementById('game-over-screen');
    const answerInput = document.getElementById('answer-input');
    const submitBtn = document.getElementById('submit-btn');
    const pauseBtn = document.getElementById('pause-btn');
    const ankiExportBtn = document.getElementById('anki-export-btn');

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

    function handleSubmit() {
        if (game.is_game_over()) {
            game.restart();
            gameOverScreen.classList.add('hidden');
            ankiExportBtn.classList.add('hidden');
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

    answerHandler = (event) => {
        if (event.key === 'Enter') {
            event.preventDefault();
            handleSubmit();
        }
    };
    answerInput.addEventListener('keydown', answerHandler);
    submitBtn.addEventListener('click', handleSubmit);

    tabKeyHandler = (event) => {
        if (event.key === 'Tab' && !game.is_game_over()) {
            event.preventDefault();
            game.pause();
        }
    };
    document.addEventListener('keydown', tabKeyHandler);

    pauseBtn.addEventListener('click', () => {
        if (!game.is_game_over()) {
            game.pause();
        }
    });

    function exportToAnki() {
        const missed_cards = game.get_missed_cards();
        if (missed_cards.length === 0) {
            alert("No missed cards to export.");
            return;
        }

        const unique_cards = missed_cards.filter((card, index, self) => 
            index === self.findIndex(c => c.raw_front === card.raw_front)
        );

        let csvContent = unique_cards.map(c => `${c.raw_front};${c.raw_back}`).join("\n");
        
        const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
        const link = document.createElement("a");
        if (link.download !== undefined) {
            const url = URL.createObjectURL(blob);
            link.setAttribute("href", url);
            link.setAttribute("download", "anki_export.csv");
            link.style.visibility = 'hidden';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        }
    }

    ankiExportBtn.addEventListener('click', exportToAnki);

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
            const missed_cards = game.get_missed_cards();
            if (missed_cards.length > 0) {
                ankiExportBtn.classList.remove('hidden');
            } else {
                ankiExportBtn.classList.add('hidden');
            }
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
    }

    startDefaultBtn.addEventListener('click', () => startGame());
    
    ankiImportInput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (!file) {
            return;
        }

        const reader = new FileReader();
        reader.onload = (e) => {
            const text = e.target.result;
            const lines = text.split('\n').filter(line => line.trim() !== '');
            const deck = lines.map(line => {
                // Ignore comment lines in Anki exports
                if (line.startsWith('#')) return null;
                const parts = line.split('\t');
                if (parts.length >= 2) {
                    // Taking first two fields, ignoring others
                    return { front: parts[0].trim(), back: parts[1].trim() };
                }
                return null;
            }).filter(Boolean);

            if (deck.length > 0) {
                showDeckConfiguration(deck);
            } else {
                alert('Could not parse deck. Make sure it is a tab-separated .txt file with "front\tback" format.');
                ankiImportInput.value = '';
            }
        };
        reader.readAsText(file);
    });

    }).catch(console.error);
}
