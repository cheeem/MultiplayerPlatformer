<script lang="ts">

	import "./app.css";

	type Player = {
		rgb: string
		width: number  
		height: number
		x_min: number
		y_min: number
	}

	let websocket: WebSocket;
	let canvas: HTMLCanvasElement;
	let cx: CanvasRenderingContext2D;
	let view_width: number;
	let view_height: number;
	let canvas_width: number;
	let canvas_height: number;

	//let players: Player[] = [];

	$: if(canvas) {

		websocket = new WebSocket("ws://localhost:3000/ws/");
		
		cx = canvas.getContext("2d");

		websocket.onmessage = render;

	}

	function render(e: { data: string }) {

		const players: Player[] = JSON.parse(e.data) as Player[];

		console.log(players);

		cx.clearRect(0, 0, canvas_width, canvas_height);

		for(let i = 0; i < players.length; i++) {

			cx.fillStyle = players[i].rgb;
			cx.fillRect(players[i].x_min, players[i].y_min, players[i].width, players[i].height);

		}

	}

	function keydown(e: KeyboardEvent) {

		switch(e.key) {
			case " ": websocket.send("j"); break;
			case "w": websocket.send("j"); break;
			case "a": websocket.send("l"); break;
			case "d": websocket.send("r"); break;
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

<svelte:window on:keydown={keydown} />
 
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
