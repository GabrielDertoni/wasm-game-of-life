import { Grid, Cell, init } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

init();

const CELL_SIZE = 2;
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const WIDTH  = 512;
const HEIGHT = 512;

const grid = new Grid(WIDTH, HEIGHT, CELL_SIZE);

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("display");
canvas.height = CELL_SIZE * HEIGHT;
canvas.width = CELL_SIZE * WIDTH;

const ctx = canvas.getContext('2d');

// document.body.addEventListener("click", () => bench());

function renderLoop() {
  grid.step();

  drawGrid();
  drawCells();

  requestAnimationFrame(renderLoop);
}

requestAnimationFrame(renderLoop);

function drawGrid() {
  ctx.beginPath();
  ctx.strokeColor = GRID_COLOR;

  ctx.moveTo(0                           , 0);
  ctx.lineTo(0                           , (CELL_SIZE + 1) * WIDTH + 1);
  ctx.lineTo((CELL_SIZE + 1) * HEIGHT + 1, (CELL_SIZE + 1) * WIDTH + 1);
  ctx.lineTo((CELL_SIZE + 1) * HEIGHT + 1, 0);
  ctx.lineTo(0                           , 0);

  /*
  // Vertical lines.
  for (let i = 0; i <= WIDTH; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * HEIGHT + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= HEIGHT; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * WIDTH + 1, j * (CELL_SIZE + 1) + 1);
  }
  */

  ctx.stroke();
}

function getIndex(row, column) {
  return row * WIDTH + column;
};

function drawCells() {
  // const cells = new Uint8Array(memory.buffer, grid.cells(), WIDTH * HEIGHT);
  // const updated_list = new Uint32Array(memory.buffer, grid.updated_list(), grid.n_updated());

  // const imageData = ctx.getImageData(0, 0, WIDTH * CELL_SIZE, HEIGHT * CELL_SIZE);
  const imageData = ctx.createImageData(WIDTH * CELL_SIZE, HEIGHT * CELL_SIZE);

  const imgBuf = new Uint8Array(memory.buffer, grid.img_buf(), WIDTH * HEIGHT * CELL_SIZE * CELL_SIZE * 4);

  imageData.data.set(imgBuf);
  ctx.putImageData(imageData, 0, 0);

  /*
  ctx.beginPath();

  for (let idx of updated_list) {
    let col = idx % WIDTH;
    let row = Math.floor(idx / WIDTH);
    ctx.fillStyle = (cells[idx] & 1) === 0
      ? DEAD_COLOR
      : ALIVE_COLOR;

    ctx.fillRect(
      col * (CELL_SIZE + 1) + 1,
      row * (CELL_SIZE + 1) + 1,
      CELL_SIZE,
      CELL_SIZE
    );
  }

  ctx.stroke();

  for (let row = 0; row < HEIGHT; row++) {
    for (let col = 0; col < WIDTH; col++) {
      const idx = getIndex(row, col);

      ctx.fillStyle = cells[idx] === Cell.Dead
        ? DEAD_COLOR
        : ALIVE_COLOR;

      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }
  */
}
