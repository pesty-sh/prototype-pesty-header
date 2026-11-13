// Dev and preview server.
//
// Process: `npm run dev` builds the wasm, then starts Vite on this port.
//
// Config:
// - The port is 43123 unless `PESTY_PORT` is set for this one command.
// - `strictPort` means Vite will not quietly pick another port if 43123 is taken.
//   Free it with Ctrl+C (it prints "Port 43123 is free.") or `npm run stop`.

import { defineConfig } from "vite";

const port = Number(process.env.PESTY_PORT || 43123);

export default defineConfig({
  server: {
    host: "0.0.0.0",
    port,
    strictPort: true,
  },
  preview: {
    host: "0.0.0.0",
    port,
    strictPort: true,
  },
});
