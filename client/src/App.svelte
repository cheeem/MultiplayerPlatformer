<script lang="ts">

	import "./app.css";

	type Entity = {
		rgb: string
		w: number  
		h: number
		x: number
		y: number
	}

	let websocket: WebSocket;
	let canvas: HTMLCanvasElement;
	let cx: CanvasRenderingContext2D;
	let view_width: number;
	let view_height: number;
	let canvas_width: number;
	let canvas_height: number;

	let holding_left: boolean = false;
	let holding_right: boolean = false;
	let holding_jump: boolean = false;

	const platforms: Entity[] = [
		{
			rgb: "white",
			w: 400.0,
			h: 30.0,
			x: 0.0,
			y: 70.0,
		},
		{
			rgb: "white",
			w: 40.0,
			h: 20.0,
			x: 0.0,
			y: 50.0,
		},
		{
			rgb: "white",
			w: 20.0,
			h: 20.0,
			x: 100.0,
			y: 50.0,
		},
	];

	$: if(canvas) {

		websocket = new WebSocket("ws://localhost:3000/ws/");
		
		cx = canvas.getContext("2d");

		websocket.onmessage = render;

	}

	function render(e: { data: string }) {

		const players: Entity[] = JSON.parse(e.data) as Entity[];

		cx.clearRect(0, 0, canvas_width, canvas_height);

		for(let i = 0; i < platforms.length; i++) {

			const platform = platforms[i];

			cx.fillStyle = platform.rgb;
			cx.fillRect(platform.x, platform.y, platform.w, platform.h);

		}

		for(let i = 0; i < players.length; i++) {

			const player = players[i];

			cx.fillStyle = player.rgb;
			cx.fillRect(player.x, player.y, player.w, player.h);

		}

	}

	function keydown(e: KeyboardEvent) {

		switch(e.key) {
			case " ":
			case "w": 
				if(holding_jump) {
					return;
				}
				websocket.send("j"); 
				holding_jump = true;
				break;
			case "a": 
				// if(holding_left) {
				// 	return;
				// }
				websocket.send("l"); 
				holding_left = true; 
				break;
			case "d": 
				// if(holding_right) {
				// 	return;
				// }
				websocket.send("r"); 
				holding_right = true;
				break;
		}

	}

	function keyup(e: KeyboardEvent) {

		switch(e.key) {
			case " ": 
			case "w": 
				holding_jump = false;
				break;
			case "a": 
				websocket.send("a"); 
				holding_left = false;
				break;
			case "d": 
				websocket.send("d"); 
				holding_right = false;
				break;
		}

	}

	// setInterval(() => {
	// 	for(let i = 0; i < players.length; i++) {
	// 		players[i].y += 1 + 0.2 * players[i].y;
	// 	}
	// 	players = players;
	// }, 50);

	// websocket = new WebSocket("ws://23.152.226.72:3000/ws/");
	// websocket = new WebSocket("ws://localhost:3000/ws/");

	// console.log("new connection");

	// websocket.onopen = function() {
	// 	console.log("connection opened");
	// 	websocket.send(username.value);
	// }

	// websocket.onclose = function() {
	// 	console.log("connection closed");
	// 	websocket = undefined;
	// }

	// websocket.onmessage = function(e) {
	// 	console.log("received message: "+e.data);
	// 	textarea.value += e.data+"\r\n";
	// }

</script>

<svelte:window on:keydown={keydown} on:keyup={keyup} />
 
<div id="view" bind:clientWidth={view_width} bind:clientHeight={view_height}> 
	<canvas bind:this={canvas} bind:clientWidth={canvas_width} bind:clientHeight={canvas_height}> 

	</canvas>
</div>


<style>

	#view {
		overflow: hidden;

		width: 100%;
    	height: 100%;
	}

	canvas {
		height: 100%;
	}

</style>
