import net from "net";
import { spawn } from "child_process";

const BASE_PORT = 1420;
const MAX_ATTEMPTS = 50;
const PORT_STEP = 2; // reserve port+1 for HMR

function checkPort(port: number, host: string): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = net.createConnection({ port, host });
    socket.once("connect", () => {
      socket.destroy();
      resolve(true);
    });
    socket.once("error", () => {
      socket.destroy();
      resolve(false);
    });
  });
}

async function isPortInUse(port: number): Promise<boolean> {
  const v4 = await checkPort(port, "127.0.0.1");
  if (v4) return true;
  const v6 = await checkPort(port, "::1");
  return v6;
}

async function findFreePort(): Promise<number> {
  for (let i = 0; i < MAX_ATTEMPTS; i++) {
    const port = BASE_PORT + i * PORT_STEP;
    const mainInUse = await isPortInUse(port);
    const hmrInUse = await isPortInUse(port + 1);
    if (!mainInUse && !hmrInUse) {
      return port;
    }
  }
  throw new Error(
    `No free port found in range ${BASE_PORT}-${BASE_PORT + MAX_ATTEMPTS * PORT_STEP}`
  );
}

const port = await findFreePort();
console.log(`Starting Tauri dev on port ${port} (HMR: ${port + 1})`);

const configOverride = JSON.stringify({
  build: {
    devUrl: `http://localhost:${port}`,
    beforeDevCommand: `vite --port ${port}`,
  },
});

const child = spawn("bun", ["run", "tauri", "dev", "--config", configOverride], {
  stdio: "inherit",
  env: process.env,
});

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => child.kill(signal));
}

child.on("exit", (code) => process.exit(code ?? 0));
