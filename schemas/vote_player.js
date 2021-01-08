// command id is 796999927782834176
// make sure to validate player being voted as part of the game.
let payload = {
    "name": "vote",
    "description": "vote that a certain player is the magician",
    "options": [
        {
            "type": 6,
            "name": "player",
            "description": "the suspected magician",
            "required": true
        }
    ]
}

// todo: raise issue about confusing error message for `default`.
