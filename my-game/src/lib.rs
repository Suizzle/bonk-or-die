// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "BONK Collecter"
    version = "1.0.0"
    author = "Turbo"
    description = "Collect the BONK before you bite the death coin! Death is temporary! Play again and again!"
    [settings]
    resolution = [256, 144]
"#}

// TODO: Get pubkey or better yet get signer from JS context

// Define the game state initialization using the turbo::init! macro
turbo::init! {
    struct GameState {
        frame: u32,
        last_munch_at: u32,
        difficulty: u32,
        dog_x: f32,
        dog_y: f32,
        dog_r: f32,
        coins: Vec<struct Coin {
            x: f32,
            y: f32,
            radius: f32,
            vel: f32,
        }>,
        death_coins: Vec<struct DeathCoin {
            x: f32,
            y: f32,
            radius: f32,
            vel: f32,
        }>,
        score: u32,
    } = {
        Self {
            frame: 0,
            last_munch_at: 0,
            difficulty: 1,
            dog_x: 128.0,
            dog_y: 125.0,
            dog_r: 8.0,
            coins: vec![],
            death_coins: vec![],
            score: 0,
        }
    }
}
// TODO: intialize counter program (devnet) PDA with score of 0 

// Implement the game loop using the turbo::go! macro
turbo::go! {
    // Load the game state
    let mut state = GameState::load();

    // Handle user input
    if gamepad(0).left.pressed() {
        state.dog_x -= 2.;
    }
    if gamepad(0).right.pressed() {
        state.dog_x += 2.;
    }

    // Generate new coins at random intervals
    if rand() % 64 == 0 {
        // Create a new coin with random attributes
        let coin = Coin {
            x: (rand() % 256) as f32,
            y: 0.0,
            radius: (rand() % 10 + 5) as f32,
            vel: (rand() % 3 + 1) as f32,
        };
        state.coins.push(coin);
    }

    // Generate new deathcoins at random intervals
    if rand() % (512 / state.difficulty) == 0 {
        // Create a new death_coin with random attributes
        let death_coin = DeathCoin {
            x: (rand() % 256) as f32,
            y: 0.0,
            radius: (rand() % 10 + 5) as f32,
            vel: (rand() % 3 + 1) as f32,
        };
        state.death_coins.push(death_coin);
    }

    // Update coin positions and check for coin collisions with the dog
    let dog_center = (state.dog_x + state.dog_r, state.dog_y + state.dog_r);
    state.coins.retain_mut(|coin| {
        coin.y += coin.vel;
        // Check for collision with the dog
        let coin_center = (coin.x + coin.radius, coin.y + coin.radius);

        // Calculate the distance between the dog and the coin
        let dx = dog_center.0 - coin_center.0;
        let dy = dog_center.1 - coin_center.1;

        let distance = (dx * dx + dy * dy).sqrt();
        let radii_sum = state.dog_r + coin.radius;
        let radii_diff = (state.dog_r - coin.radius).abs();

        if radii_diff <= distance && distance <= radii_sum {
            // Dog caught the coin
            state.score += 1;
            state.difficulty +=1;
            // TODO: Play coin sound
            // TODO: Send the new score to the devnet counter program to make the game fully on chain
            state.last_munch_at = state.frame;
            false // Remove the coin from the game
        } else if coin.y < 144. + (coin.radius * 2.) {
            true // Keep the coin in the game if it's within the screen
        } else {
            false // Remove the coin if it's off-screen
        }
    });

    // Update deathcoin positions and check for coin collisions with the dog
    let dog_center = (state.dog_x + state.dog_r, state.dog_y + state.dog_r);
    state.death_coins.retain_mut(|death_coin| {
        death_coin.y += death_coin.vel;
        // Check for collision with the dog
        let death_coin_center = (death_coin.x + death_coin.radius, death_coin.y + death_coin.radius);

        // Calculate the distance between the dog and the coin
        let dx = dog_center.0 - death_coin_center.0;
        let dy = dog_center.1 - death_coin_center.1;

        let distance = (dx * dx + dy * dy).sqrt();
        let radii_sum = state.dog_r + death_coin.radius;
        let radii_diff = (state.dog_r - death_coin.radius).abs();

        if radii_diff <= distance && distance <= radii_sum {
            // Dog caught the death_coin
            // TODO: end the game here: 
            // make canvas all black, play death sound and pass score to frontend
            state.last_munch_at = state.frame;
            false // Remove the coin from the game
        } else if death_coin.y < 144. + (death_coin.radius * 2.) {
            true // Keep the coin in the game if it's within the screen
        } else {
            false // Remove the coin if it's off-screen
        }
    });

    // Set the background color
    clear(0x00ffffff);

    // Draw a tiled background of moving sprites
    let frame = (state.frame as i32) / 2;
    for col in 0..9 {
        for row in 0..6 {
            let x = col * 32;
            let y = row * 32;
            let x = ((x + frame) % (272 + 16)) - 32;
            let y = ((y + frame) % (144 + 16)) - 24;
            sprite!("heart", x = x, y = y);
        }
    }

    // Draw a speech bubble when the dog eats a coin
    if state.frame >= 64 && state.frame.saturating_sub(state.last_munch_at) <= 60 {
        rect!(w = 30, h = 10, x = state.dog_x as i32 + 32, y = state.dog_y as i32);
        circ!(d = 10, x = state.dog_x as i32 + 28, y = state.dog_y as i32);
        rect!(w = 10, h = 5, x = state.dog_x as i32 + 28, y = state.dog_y as i32 + 5);
        circ!(d = 10, x = state.dog_x as i32 + 56, y = state.dog_y as i32);
        text!("BONK!", x = state.dog_x as i32 + 33, y = state.dog_y as i32 + 3, font = Font::S, color = 0x000000ff);
    }

    // Draw the dog
    sprite!("munch_dog", x = (state.dog_x - state.dog_r) as i32, y = (state.dog_y - 16.) as i32, fps = fps::FAST);

    // Draw the coins
    for coin in &state.coins {
        circ!(x = coin.x as i32, y = coin.y as i32 + 1, d = (coin.radius + 2.) as u32, fill = 0x000000aa); // Render the coins
        circ!(x = coin.x as i32, y = coin.y as i32, d = (coin.radius + 1.) as u32, fill = 0xefff00ff); // Render the coins
        circ!(x = coin.x as i32, y = coin.y as i32, d = coin.radius as u32, fill = 0xf1c232ff); // Render the coins
    }

    for death_coin in &state.death_coins {
        circ!(x = death_coin.x as i32, y = death_coin.y as i32 + 1, d = (death_coin.radius + 2.) as u32, fill = 0x000000aa); // Render the deathcoins
        circ!(x = death_coin.x as i32, y = death_coin.y as i32, d = (death_coin.radius + 1.) as u32, fill = 0xea9999ff); // Render the deathcoins
        circ!(x = death_coin.x as i32, y = death_coin.y as i32, d = death_coin.radius as u32, fill = 0x990000ff); // Render the deathcoins
    }

    // Draw the score
    text!(&format!("Score: {}", state.score), x = 10, y = 10, font = Font::L, color = 0xffffffff); // Render the score

    // Uncomment to print game state for debugging
    // text!(&format!("{:#?}", state), y = 24);

    // Save game state for the next frame
    state.frame += 1;
    state.save();
}
