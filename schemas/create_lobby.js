// command id is 796995810038382642
let payload = {
	"name": "create",
	"description": "create a lobby",
	"options": [
        // this option will only work if the executor has... MANAGE_CHANNEL... ?
		{
			"type": 5,
			"name": "hijack",
			"description": "whether to take over the current channel's lobby"
		}
	]
}
