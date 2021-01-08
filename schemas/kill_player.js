// command id is 796999207046742027
// make sure to validate player is in game
// and that runner is magician
let payload = {
    "name": "experiment",
    "description": "experiment with a player, possibly turning them undead",
    "options": [
        {
            "type": 6,
            "name": "player",
            "description": "the player to experiment with",
            "required": true
        }
    ]
}

// todo: raise issue about confusing error message for `default`.
